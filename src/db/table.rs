use std::rc::Rc;
use std::cell::RefCell; 

use crate::db::error::{DbError, DbResult};
use crate::db::pager::{Pager, TABLE_MAX_PAGES};
use crate::db::tree::{initialize_leaf_node, set_node_root};
use libc::{self, c_void};

#[derive(Default)]
pub struct Table {
    pub pager: Rc<RefCell<Pager>>,
    pub root_page_num: usize,
}

impl Table {
    pub fn db_open(filename: &str) -> DbResult<Self> {
        let mut pager = Pager::new(filename)?;

        if pager.num_pages == 0 {
            let root_node: *mut c_void = pager.get_page(0)?;
            initialize_leaf_node(root_node);
            set_node_root(root_node, true);
        }

        return Ok(Self {
            pager: Rc::new(RefCell::new(pager)),
            root_page_num: 0,
        });
    }

    pub fn db_close(&mut self) -> DbResult<()> {
        let pager = self.pager.clone();

        for i in 0..pager.borrow().num_pages {
            if pager.borrow().pages[i].is_null() {
                continue;
            }
            let _ = pager.borrow_mut().pager_flush(i)?;
            unsafe {
                libc::free(pager.borrow_mut().pages[i]);
            }
            pager.borrow_mut().pages[i] = std::ptr::null_mut::<c_void>();
        }

        let result = unsafe {libc::close(self.pager.borrow().file_descripter)};
        if result == -1 {
            return Err(DbError::Other("Error closing db file.".to_owned()));
        }

        for i in 0..TABLE_MAX_PAGES {
            let page: *mut c_void = self.pager.borrow().pages[i];
            if !page.is_null() {
                unsafe{libc::free(page)};
                self.pager.borrow_mut().pages[i] = std::ptr::null_mut::<c_void>();
            }
        }
         Ok(())
    }
}
