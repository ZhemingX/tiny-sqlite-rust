use core::num;
use std::rc::Rc;

use libc::c_void;

use super::table::Table;
use crate::db::error::{DbError, DbResult};
use crate::db::tree::*;

pub struct Cursor {
    table: Rc<Table>,
    page_num: usize,
    cell_num: usize,
    end_of_table: bool,
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

    fn cursor_value(&self) -> DbResult<*mut c_void> {
        let page_num = self.page_num;
        let page: *mut c_void = self.table.pager.borrow_mut().get_page(page_num)?;
        Ok(leaf_node_value(page, self.cell_num))
    }

    fn cursor_advance(&mut self) -> DbResult<()> {
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
}
