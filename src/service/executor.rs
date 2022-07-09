use std::rc::Rc;
use std::cell::RefCell;

use libc::c_void;

use super::{Statement, StatementType};
use crate::db::table::Table;
use crate::db::error::{DbError, DbResult};
use crate::service::Row;
use crate::db::cursor::Cursor;
use crate::db::tree::*;

pub enum ExecuteResult {
    ExecuteSuccess,
    ExecuteDuplicateKey,
}

pub struct Executor {}

impl Executor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn execute_insert(&self, stmt: &Statement,  table: Rc<Table>) -> DbResult<ExecuteResult>  {
        let row_to_insert = &stmt.row_to_insert;
        let key_to_insert: u32 = row_to_insert.id;
        let cursor = Cursor::table_find(table.clone(), key_to_insert)?;

        let node = table.pager.borrow_mut().get_page(cursor.page_num)?;
        let num_cells = unsafe{*leaf_node_key(node, cursor.cell_num) as usize };

        if cursor.cell_num < num_cells {
            let key_at_index = unsafe {
                *leaf_node_key(node, cursor.cell_num)
            };
            if key_at_index == key_to_insert {
                return Ok(ExecuteResult::ExecuteDuplicateKey);
            }
        }

        cursor.leaf_node_insert( key_to_insert, Rc::new(RefCell::new(stmt.row_to_insert)))?;

        Ok(ExecuteResult::ExecuteSuccess)
    }

    pub fn execute_select(&self, table: Rc<Table>) -> DbResult<ExecuteResult>  {
        let mut cursor = Cursor::table_start(table.clone())?;

        let mut row = Row::default();
        while !cursor.end_of_table {
            row.deserialize_row(cursor.cursor_value()? as *const c_void);
            println!("{}", row);
            cursor.cursor_advance()?;
        }

        Ok(ExecuteResult::ExecuteSuccess)
    }

    pub fn execute_statement(&self, stmt: &Statement,  table: Rc<Table>) -> DbResult<ExecuteResult> {
        match stmt.stmt_type {
            StatementType::StatementInsert => self.execute_insert(stmt, table),
            StatementType::StatementSelect => self.execute_select(table),
        }
    }
}