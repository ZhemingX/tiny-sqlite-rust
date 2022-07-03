use std::{mem, num};

use super::error::{DbError, DbResult};
use super::pager::{PAGE_SIZE, Pager};
use crate::service::Row;

use libc::c_void;

macro_rules! field_size {
    ($t:ident :: $field:ident) => {{
        let m = core::mem::MaybeUninit::<$t>::uninit();
        let p = unsafe { core::ptr::addr_of!((*(&m as *const _ as *const $t)).$field) };

        const fn size_of_raw<T>(_: *const T) -> usize {
            core::mem::size_of::<T>()
        }
        size_of_raw(p)
    }};
}

pub enum NodeType {
    NodeInternal,
    NodeLeaf,
}

pub const ID_SIZE: usize = field_size!(Row::id);
pub const USERNAME_SIZE: usize = field_size!(Row::username);
pub const EMAIL_SIZE: usize = field_size!(Row::email);
pub const ID_OFFSET: usize = 0;
pub const USERNAME_OFFSET: usize = ID_OFFSET + ID_SIZE;
pub const EMAIL_OFFSET: usize = USERNAME_OFFSET + USERNAME_SIZE;
pub const ROW_SIZE: usize = ID_SIZE + USERNAME_SIZE + EMAIL_SIZE;

/*
 * Common Node Header Layout
 */
const NODE_TYPE_SIZE: usize = mem::size_of::<u8>();
const NODE_TYPE_OFFSET: usize = 0;
const IS_ROOT_SIZE: usize = mem::size_of::<u8>();
const IS_ROOT_OFFSET: usize = NODE_TYPE_SIZE;
const PARENT_POINTER_SIZE: usize = mem::size_of::<u32>();
const PARENT_POINTER_OFFSET: usize = IS_ROOT_OFFSET + IS_ROOT_SIZE;
const COMMON_NODE_HEADER_SIZE: usize = NODE_TYPE_SIZE + IS_ROOT_SIZE + PARENT_POINTER_SIZE;

/*
 * Internal Node Header Layout
 */
const INTERNAL_NODE_NUM_KEYS_SIZE: usize = mem::size_of::<u32>();
const INTERNAL_NODE_NUM_KEYS_OFFSET: usize = COMMON_NODE_HEADER_SIZE;
const INTERNAL_NODE_RIGHT_CHILD_SIZE: usize = mem::size_of::<u32>();
const INTERNAL_NODE_RIGHT_CHILD_OFFSET: usize =
    INTERNAL_NODE_NUM_KEYS_OFFSET + INTERNAL_NODE_NUM_KEYS_SIZE;
const INTERNAL_NODE_HEADER_SIZE: usize =
    COMMON_NODE_HEADER_SIZE + INTERNAL_NODE_NUM_KEYS_SIZE + INTERNAL_NODE_RIGHT_CHILD_SIZE;

/*
 * Internal Node Body Layout
 */
const INTERNAL_NODE_KEY_SIZE: usize = mem::size_of::<u32>();
const INTERNAL_NODE_CHILD_SIZE: usize = mem::size_of::<u32>();
const INTERNAL_NODE_CELL_SIZE: usize = INTERNAL_NODE_CHILD_SIZE + INTERNAL_NODE_KEY_SIZE;
/* Keep this small for testing */
const INTERNAL_NODE_MAX_CELLS: usize = 3;

/*
 * Leaf Node Header Layout
 */
const LEAF_NODE_NUM_CELLS_SIZE: usize = mem::size_of::<u32>();
const LEAF_NODE_NUM_CELLS_OFFSET: usize = COMMON_NODE_HEADER_SIZE;
const LEAF_NODE_NEXT_LEAF_SIZE: usize = mem::size_of::<u32>();
const LEAF_NODE_NEXT_LEAF_OFFSET: usize = LEAF_NODE_NUM_CELLS_OFFSET + LEAF_NODE_NUM_CELLS_SIZE;
const LEAF_NODE_HEADER_SIZE: usize =
    COMMON_NODE_HEADER_SIZE + LEAF_NODE_NUM_CELLS_SIZE + LEAF_NODE_NEXT_LEAF_SIZE;

/*
 * Leaf Node Body Layout
 */
const LEAF_NODE_KEY_SIZE: usize = mem::size_of::<u32>();
const LEAF_NODE_KEY_OFFSET: usize = 0;
const LEAF_NODE_VALUE_SIZE: usize = ROW_SIZE;
const LEAF_NODE_VALUE_OFFSET: usize = LEAF_NODE_KEY_OFFSET + LEAF_NODE_KEY_SIZE;
const LEAF_NODE_CELL_SIZE: usize = LEAF_NODE_KEY_SIZE + LEAF_NODE_VALUE_SIZE;
const LEAF_NODE_SPACE_FOR_CELLS: usize = PAGE_SIZE - LEAF_NODE_HEADER_SIZE;
const LEAF_NODE_MAX_CELLS: usize = LEAF_NODE_SPACE_FOR_CELLS / LEAF_NODE_CELL_SIZE;
const LEAF_NODE_RIGHT_SPLIT_COUNT: usize = (LEAF_NODE_MAX_CELLS + 1) / 2;
const LEAF_NODE_LEFT_SPLIT_COUNT: usize = (LEAF_NODE_MAX_CELLS + 1) - LEAF_NODE_RIGHT_SPLIT_COUNT;

// methods for low-level b-tree implementation

// ----------- print -----------------//
fn print_constants() {
    println!("ROW_SIZE: {}", ROW_SIZE);
    println!("COMMON_NODE_HEADER_SIZE: {}", COMMON_NODE_HEADER_SIZE);
    println!("LEAF_NODE_HEADER_SIZE: {}", LEAF_NODE_HEADER_SIZE);
    println!("LEAF_NODE_CELL_SIZE: {}", LEAF_NODE_CELL_SIZE);
    println!("LEAF_NODE_SPACE_FOR_CELLS: {}", LEAF_NODE_SPACE_FOR_CELLS);
    println!("LEAF_NODE_MAX_CELLS: {}", LEAF_NODE_MAX_CELLS);
}

fn indent(level: u32) {
    for i in 0..level {
        print!("  ");
    }
}

fn print_tree(pager: &mut Pager, page_num: usize, indentation_level: u32) -> DbResult<()> {
    let node = pager.get_page(page_num)?;
    let mut num_keys: u32 = 0;
    let mut child: u32 = 0;

    match get_node_type(node) {
        NodeType::NodeLeaf => {
            num_keys = unsafe {
                *leaf_node_num_cells(node)
            };
            indent(indentation_level);
            println!("- leaf (size {})", num_keys);
            for i in 0..num_keys {
                indent(indentation_level + 1);
                unsafe{
                    println!("- {}", *leaf_node_key(node, i as usize));
                }
            }
            return Ok(());
        },
        NodeType::NodeInternal => {
            num_keys = unsafe{
                *internal_node_num_keys(node)
            };
            indent(indentation_level);
            println!("- internal (size {})", num_keys);
            for i in 0..num_keys {
                child = unsafe {
                    *internal_node_child(node, i as usize)?
                };
                
                let print_res = print_tree(pager, child as usize, indentation_level + 1);
                if print_res.is_err() {
                    return print_res;
                }

                indent(indentation_level + 1);
                unsafe {
                    println!("- key {}", *internal_node_key(node, i as usize));
                }
            }
            child = unsafe {
                *internal_node_right_child(node)
            };
            
            return print_tree(pager, child as usize, indentation_level + 1);  
        }
    }
}

// ----------- print -----------------//

pub fn get_node_type(node: *const c_void) -> NodeType {
    unsafe {
        let node_type_ptr = (node as *const u8)
            .offset(NODE_TYPE_OFFSET as isize);
        if *node_type_ptr == 0 {
            NodeType::NodeInternal
        } else {
            NodeType::NodeLeaf
        }
    }
}

fn set_node_type(node: *mut c_void, n_type: NodeType) {
    let type_num: u8 = match n_type {
        NodeType::NodeInternal => 0,
        NodeType::NodeLeaf => 1,
    };
    unsafe {
        let node_type_ptr = (node as *const u8)
            .offset(NODE_TYPE_OFFSET as isize) 
            as *mut u8;
        *node_type_ptr = type_num;
    }
}

fn is_node_root(node: *const c_void) -> bool {
    unsafe {
        let node_root_ptr = (node as *const u8)
            .offset(IS_ROOT_OFFSET as isize);
        return *node_root_ptr != 0;
    }
}

pub fn set_node_root(node: *mut c_void, is_root: bool) {
    unsafe {
        let node_root_ptr = (node as *const u8)
            .offset(NODE_TYPE_OFFSET as isize) 
            as *mut u8;
        *node_root_ptr = is_root as u8;
    }
}

fn node_parent(node: *mut c_void) -> *mut u32 {
    unsafe {
        return (node as *const u8)
            .offset(PARENT_POINTER_OFFSET as isize) 
            as *mut u32;
    }
}

fn internal_node_num_keys(node: *mut c_void) -> *mut u32 {
    unsafe {
        return (node as *const u8)
            .offset(INTERNAL_NODE_NUM_KEYS_OFFSET as isize) 
            as *mut u32;
    }
}

fn internal_node_right_child(node: *mut c_void) -> *mut u32 {
    unsafe {
        return (node as *const u8)
            .offset(INTERNAL_NODE_RIGHT_CHILD_OFFSET as isize) 
            as *mut u32;
    }
}

fn internal_node_cell(node: *mut c_void, cell_num: usize) -> *mut u32 {
    unsafe {
        return (node as *const u8)
            .offset(
            INTERNAL_NODE_HEADER_SIZE as isize + (cell_num * INTERNAL_NODE_CELL_SIZE
            ) as isize,
        ) as *mut u32;
    }
}

pub fn internal_node_child(node: *mut c_void, child_num: usize) -> DbResult<*mut u32> {
    unsafe {
        let num_keys = *internal_node_num_keys(node) as usize;
        if child_num > num_keys {
            return Err(DbError::Other(format!(
                "Tried to access child_num {} > num_keys {}",
                child_num, num_keys
            )));
        } else if child_num == num_keys {
            return Ok(internal_node_right_child(node));
        } else {
            return Ok(internal_node_cell(node, child_num));
        }
    }
}

fn internal_node_key(node: *mut c_void, key_num: usize) -> *mut u32 {
    unsafe {
        let internal_node_key_ptr = (internal_node_cell(node, key_num) as *const u8)
            .offset(INTERNAL_NODE_CHILD_SIZE as isize)
            as *mut u32;
        internal_node_key_ptr
    }
}

pub fn leaf_node_num_cells(node: *mut c_void) -> *mut u32 {
    unsafe {
        return (node as *const u8)
            .offset(LEAF_NODE_NUM_CELLS_OFFSET as isize) 
            as *mut u32;
    }
}

pub fn leaf_node_next_leaf(node: *mut c_void) -> *mut u32 {
    unsafe {
        return (node as *const u8)
            .offset(LEAF_NODE_NEXT_LEAF_OFFSET as isize) 
            as *mut u32;
    }
}

fn leaf_node_cell(node: *mut c_void, cell_num: usize) -> *mut c_void {
    unsafe {
        return (node as *const u8)
            .offset(LEAF_NODE_HEADER_SIZE as isize + (cell_num * LEAF_NODE_CELL_SIZE) as isize)
            as *mut c_void;
    }
}

pub fn leaf_node_key(node: *mut c_void, cell_num: usize) -> *mut u32 {
    unsafe {
        return leaf_node_cell(node, cell_num)
            as *mut u32;
    }
  }
  
pub fn leaf_node_value(node: *mut c_void, cell_num: usize) -> *mut c_void {
    unsafe {
        return (leaf_node_cell(node, cell_num) as *const u8)
            .offset(LEAF_NODE_KEY_SIZE as isize)
            as *mut c_void;
    }
}

fn get_node_max_key(node: *mut c_void) -> u32 {
    unsafe {
        match get_node_type(node) {
            NodeType::NodeInternal => *internal_node_key(node, *internal_node_num_keys(node) as usize - 1),
            NodeType::NodeLeaf => *leaf_node_key(node, *leaf_node_num_cells(node) as usize - 1),
        }
    }
}

pub fn initialize_leaf_node(node: *mut c_void) {
    set_node_type(node, NodeType::NodeLeaf);
    set_node_root(node, false);
    unsafe {
        *leaf_node_num_cells(node) = 0;
        *leaf_node_next_leaf(node) = 0; // 0 represents no sibling
    } 
}

fn initialize_internal_node(node: *mut c_void) {
    set_node_type(node, NodeType::NodeInternal);
    set_node_root(node, false);
    unsafe {
        *internal_node_num_keys(node) = 0;
    }
}

pub fn internal_node_find_child(node: *mut c_void, key: u32) -> u32 {
    /*
    Return the index of the child which should contain
    the given key.
    */
  
    let num_keys = unsafe {
        *internal_node_num_keys(node)
    };

    /* Binary search */
    let mut min_index: u32 = 0;
    let mut max_index: u32 = num_keys; /* there is one more child than key */
  
    while min_index != max_index {
      let index: u32 = min_index + (max_index - min_index) / 2;
      let key_to_right: u32 = unsafe{
          *internal_node_key(node, index as usize)
        };
      if key_to_right >= key {
        max_index = index;
      } else {
        min_index = index + 1;
      }
    }
    min_index
  }