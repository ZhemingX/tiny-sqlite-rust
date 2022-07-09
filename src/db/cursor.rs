use core::num;
use std::rc::Rc;
use std::cell::RefCell;

use libc::c_void;

use super::table::Table;
use crate::db::error::{DbError, DbResult};
use crate::db::tree::*;
use crate::service::Row;

pub struct Cursor {
    table: Rc<Table>,
    pub page_num: usize,
    pub cell_num: usize,
    pub end_of_table: bool,
}

impl Cursor {
    fn leaf_node_find(table: Rc<Table>, page_num: usize, key: u32) -> DbResult<Self> {
        let node = table.pager.borrow_mut().get_page(page_num)?;
        let num_cells = unsafe{
            *leaf_node_num_cells(node)
        };

        let mut cursor = Cursor {
            table: table,
            page_num: page_num,
            cell_num: 0,
            end_of_table:false,
        };

        // Binary search
        let mut min_index: u32 = 0;
        let mut one_past_max_index: u32 = num_cells;

        while one_past_max_index != min_index {
            let index: u32 = min_index + (one_past_max_index - min_index) / 2;
            let key_at_index: u32 = unsafe {
                *leaf_node_key(node, index as usize)
            };
            if key == key_at_index {
            cursor.cell_num = index as usize;
            return Ok(cursor);
            }
            if key < key_at_index {
            one_past_max_index = index;
            } else {
            min_index = index + 1;
            }
        }

        cursor.cell_num = min_index as usize;

        Ok(cursor)
    }

    fn internal_node_find(table: Rc<Table>, page_num: usize, key: u32) -> DbResult<Self> {
        let node = table.pager.borrow_mut().get_page(page_num)?;
      
        let child_index: u32 = internal_node_find_child(node, key);
        let child_num: u32 = unsafe {
            *internal_node_child(node, child_index as usize)?
        };
        let child = table.pager.borrow_mut().get_page(child_num as usize)?;
        match get_node_type(child) {
            NodeType::NodeLeaf => Cursor::leaf_node_find(table, page_num, key),
            NodeType::NodeInternal => Cursor::internal_node_find(table, child_num as usize, key)
        }
    }

    pub fn table_find(table: Rc<Table>, key: u32) -> DbResult<Self> {
        let root_page_num = table.root_page_num as usize;
        let root_node = table.pager.borrow_mut().get_page(root_page_num)?;

        match get_node_type(root_node as *const c_void) {
            NodeType::NodeLeaf => Cursor::leaf_node_find(table, root_page_num, key),
            NodeType::NodeInternal => Cursor::internal_node_find(table, root_page_num, key)
        }
    }

    pub fn table_start(table: Rc<Table>) -> DbResult<Self> {
        let mut cursor = Cursor::table_find(table.clone(), 0)?;

        let node = table.pager.borrow_mut().get_page(cursor.page_num)?;
        let num_cells: u32 = unsafe{
            *leaf_node_num_cells(node)
        };
        cursor.end_of_table = num_cells == 0;

        Ok(cursor)
    }

    pub fn cursor_value(&self) -> DbResult<*mut c_void> {
        let page_num = self.page_num;
        let page: *mut c_void = self.table.pager.borrow_mut().get_page(page_num)?;
        Ok(leaf_node_value(page, self.cell_num))
    }

    pub fn cursor_advance(&mut self) -> DbResult<()> {
        let page_num = self.page_num;
        let node: *mut c_void = self.table.pager.borrow_mut().get_page(page_num)?;

        self.cell_num += 1;
        if self.cell_num >= (unsafe{ *leaf_node_num_cells(node) as usize }) {
            let next_page_num = unsafe { *leaf_node_next_leaf(node) as usize };
            if next_page_num == 0 {
                self.end_of_table = true;
            } else {
                self.page_num = next_page_num;
                self.cell_num = 0;
            }
        }

        Ok(())
    }

    pub fn leaf_node_split_and_insert(&self, key: u32, value: Rc<RefCell<Row>>) -> DbResult<()> {
        /*
        Create a new node and move half the cells over.
        Insert the new value in one of the two nodes.
        Update parent or create a new parent.
        */
      
        let old_node = self.table.pager.borrow_mut().get_page(self.page_num)?;
        let old_max = get_node_max_key(old_node);
        let new_page_num = self.table.pager.borrow_mut().get_unused_page_num();
        let new_node = self.table.pager.borrow_mut().get_page(new_page_num)?;
        initialize_leaf_node(new_node);
        unsafe{
            *node_parent(new_node) = *node_parent(old_node);
            *leaf_node_next_leaf(new_node) = *leaf_node_next_leaf(old_node);
            *leaf_node_next_leaf(old_node) = new_page_num as u32;
        }
        /*
        All existing keys plus new key should should be divided
        evenly between old (left) and new (right) nodes.
        Starting from the right, move each key to correct position.
        */
        for i in (0..(LEAF_NODE_MAX_CELLS+1)).rev() {
          let destination_node =  if i >= LEAF_NODE_LEFT_SPLIT_COUNT {
              new_node
          } else {
              old_node
          };

          let index_within_node = i % LEAF_NODE_LEFT_SPLIT_COUNT;
          let destination = leaf_node_cell(destination_node, index_within_node);
      
          if i == self.cell_num {
            value.borrow().serialize_row(
                          leaf_node_value(destination_node, index_within_node));
            unsafe {
                *leaf_node_key(destination_node, index_within_node) = key;
            }
          } else if i > self.cell_num {
            unsafe {libc::memcpy(destination, leaf_node_cell(old_node, i - 1), LEAF_NODE_CELL_SIZE);}
          } else {
            unsafe {libc::memcpy(destination, leaf_node_cell(old_node, i), LEAF_NODE_CELL_SIZE);}
          }
        }
      
        /* Update cell count on both leaf nodes */
        unsafe {
            *(leaf_node_num_cells(old_node)) = LEAF_NODE_LEFT_SPLIT_COUNT as u32;
            *(leaf_node_num_cells(new_node)) = LEAF_NODE_RIGHT_SPLIT_COUNT as u32; 
        }
      
        if is_node_root(old_node as *const c_void) {
          return self.table.create_new_root(new_page_num);
        } else {
            let parent_page_num = unsafe{*node_parent(old_node)};
            let new_max = get_node_max_key(old_node);
            let parent = self.table.pager.borrow_mut().get_page(parent_page_num as usize)?;
        
            update_internal_node_key(parent, old_max, new_max);
            self.table.internal_node_insert(parent_page_num as usize, new_page_num)?;
        }

        Ok(())
    }

    pub fn leaf_node_insert(&self, key: u32, value: Rc<RefCell<Row>>) -> DbResult<()> {
        let node = self.table.pager.borrow_mut().get_page(self.page_num)?;
      
        let num_cells = unsafe{*leaf_node_num_cells(node)};
        if num_cells as usize >= LEAF_NODE_MAX_CELLS {
          // Node full
          return self.leaf_node_split_and_insert(key, value);
        }
      
        if self.cell_num < num_cells as usize {
          // Make room for new cell
          for i in ((self.cell_num + 1)..(num_cells as usize + 1)).rev() {
              unsafe{
                libc::memcpy(leaf_node_cell(node, i), leaf_node_cell(node, i-1), LEAF_NODE_CELL_SIZE);
              }
          }
        }
        
        unsafe {
            *(leaf_node_num_cells(node)) += 1;
            *(leaf_node_key(node, self.cell_num)) = key;
        }
        value.borrow().serialize_row(leaf_node_value(node, self.cell_num));

        Ok(())
      }
}
