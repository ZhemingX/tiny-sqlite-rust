use crate::db::error::DbError;
use crate::db::pager::{DbResult, Pager};

use libc::c_void;

#[derive(Default)]
pub struct Table {
    pager: Pager,
    root_page_num: usize,
}

impl Table {
    pub fn db_open(filename: &str) -> DbResult<Self> {
        let mut pager = Pager::new(filename)?;

        if pager.num_pages == 0 {
            let root_node: *mut c_void = pager.get_page(0)?;
        }

        return Ok(Self {
            pager: pager,
            root_page_num: 0,
        });
    }
}
