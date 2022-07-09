use std::rc::Rc;

use crate::db::tree::*;
use crate::db::table::Table;
use crate::db::error::{DbError, DbResult};

pub enum MetaCommandResult {
    MetaCmdSuccess,
    MetaCmdExit,
    MetaCmdUnrecognizedCmd,
}

pub struct MetaCommandService {}

impl MetaCommandService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn do_meta_command(&self, cmd: &str, table: Rc<Table>) -> DbResult<MetaCommandResult> {
        match cmd {
            ".exit;" => {
                table.db_close()?;
                Ok(MetaCommandResult::MetaCmdExit)
            },
            ".btree;" => {
                println!("print btree\n");
                print_tree(&mut table.pager.borrow_mut(), 0, 0)?;
                Ok(MetaCommandResult::MetaCmdSuccess)
            }
            ".constants;" => {
                println!("print constants\n");
                print_constants();
                Ok(MetaCommandResult::MetaCmdSuccess)
            }
            _ => {
                // unrecognized command
                Ok(MetaCommandResult::MetaCmdUnrecognizedCmd)
            }
        }
    }
}
