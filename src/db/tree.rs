use std::mem;

use super::pager::PAGE_SIZE;
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

const ID_SIZE: usize = field_size!(Row::id);
const USERNAME_SIZE: usize = field_size!(Row::username);
const EMAIL_SIZE: usize = field_size!(Row::email);
const ID_OFFSET: usize = 0;
const USERNAME_OFFSET: usize = ID_OFFSET + ID_SIZE;
const EMAIL_OFFSET: usize = USERNAME_OFFSET + USERNAME_SIZE;
const ROW_SIZE: usize = ID_SIZE + USERNAME_SIZE + EMAIL_SIZE;

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

fn get_node_type(node: *const c_void) -> NodeType {
    unsafe {
        let node_type_ptr = (node as *const u8).offset(NODE_TYPE_OFFSET as isize);
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
        let node_type_ptr = (node as *const u8).offset(NODE_TYPE_OFFSET as isize) as *mut u8;
        *node_type_ptr = type_num;
    }
}

fn is_node_root(node: *const c_void) -> bool {
    unsafe {
        let node_root_ptr = (node as *const u8).offset(IS_ROOT_OFFSET as isize);
        return *node_root_ptr != 0;
    }
}

fn set_node_root(node: *mut c_void, is_root: bool) {
    unsafe {
        let node_root_ptr = (node as *const u8).offset(NODE_TYPE_OFFSET as isize) as *mut u8;
        *node_root_ptr = is_root as u8;
    }
}

fn node_parent(node: *mut c_void) -> *mut u32 {
    unsafe {
        return (node as *const u8).offset(PARENT_POINTER_OFFSET as isize) as *mut u32;
    }
}

fn internal_node_num_keys(node: *mut c_void) -> *mut u32 {
    unsafe {
        return (node as *const u8).offset(INTERNAL_NODE_NUM_KEYS_OFFSET as isize) as *mut u32;
    }
}

fn internal_node_right_child(node: *mut c_void) -> *mut u32 {
    unsafe {
        return (node as *const u8).offset(INTERNAL_NODE_RIGHT_CHILD_OFFSET as isize) as *mut u32;
    }
}

fn internal_node_cell(node: *mut c_void, cell_num: usize) -> *mut u32 {
    unsafe {
        return (node as *const u8).offset(
            INTERNAL_NODE_HEADER_SIZE as isize + (cell_num * INTERNAL_NODE_CELL_SIZE) as isize,
        ) as *mut u32;
    }
}
