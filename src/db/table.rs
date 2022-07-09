use std::rc::Rc;
use std::cell::RefCell; 

use crate::db::error::{DbError, DbResult};
use crate::db::pager::{Pager, TABLE_MAX_PAGES, PAGE_SIZE};
use crate::db::tree::*;
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

    pub fn db_close(&self) -> DbResult<()> {
        let pager = self.pager.clone();
        let num_pages = pager.borrow_mut().num_pages;

        for i in 0..num_pages {
            if pager.borrow_mut().pages[i].is_null() {
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
            let page: *mut c_void = self.pager.borrow_mut().pages[i];
            if !page.is_null() {
                unsafe{libc::free(page)};
                self.pager.borrow_mut().pages[i] = std::ptr::null_mut::<c_void>();
            }
        }
         Ok(())
    }

    pub fn create_new_root(&self, right_child_page_num: usize) -> DbResult<()> {
        /*
        Handle splitting the root.
        Old root copied to new page, becomes left child.
        Address of right child passed in.
        Re-initialize root page to contain the new root node.
        New root node points to two children.
        */
        let root = self.pager.borrow_mut().get_page(self.root_page_num)?;
        let right_child = self.pager.borrow_mut().get_page(right_child_page_num)?;
        let left_child_page_num: usize = self.pager.borrow_mut().get_unused_page_num();
        let left_child = self.pager.borrow_mut().get_page(left_child_page_num)?;
      
        /* Left child has data copied from old root */
        unsafe{
            libc::memcpy(left_child, root as *const c_void, PAGE_SIZE);
        }
        set_node_root(left_child, false);
      
        /* Root node is a new internal node with one key and two children */
        initialize_internal_node(root);
        set_node_root(root, true);
        unsafe {
            *internal_node_num_keys(root) = 1;
            *(internal_node_child(root, 0)?) = left_child_page_num as u32;
            let left_child_max_key: u32 = get_node_max_key(left_child);
            *internal_node_key(root, 0) = left_child_max_key;
            *internal_node_right_child(root) = right_child_page_num as u32;
            *node_parent(left_child) = self.root_page_num as u32;
            *node_parent(right_child) = self.root_page_num as u32;
        }

        Ok(())
    }
    
    pub fn internal_node_insert(&self, parent_page_num: usize,
        child_page_num: usize) -> DbResult<()> {
        /*
        Add a new child/key pair to parent that corresponds to child
        */
        let parent =self.pager.borrow_mut().get_page(parent_page_num as usize)?;
        let child = self.pager.borrow_mut().get_page(child_page_num as usize)?;
        let child_max_key: u32 = get_node_max_key(child);
        let index: u32 = internal_node_find_child(parent, child_max_key);
        
        let original_num_keys: u32 = unsafe {
                *internal_node_num_keys(parent)
            };
            unsafe {
                *internal_node_num_keys(parent) = original_num_keys + 1;
            }

        if (original_num_keys as usize >= INTERNAL_NODE_MAX_CELLS) {
            return Err(DbError::Other("Need to implement splitting internal node".to_string()));
        }

        let right_child_page_num: u32 = unsafe {
            *internal_node_right_child(parent)
        };
        let right_child = self.pager.borrow_mut().get_page(right_child_page_num as usize)?;

        if child_max_key > get_node_max_key(right_child) {
            /* Replace right child */
            unsafe {
                *(internal_node_child(parent, original_num_keys as usize)?) = right_child_page_num;
                *internal_node_key(parent, original_num_keys as usize) =
                get_node_max_key(right_child);
                *internal_node_right_child(parent) = child_page_num as u32;
            }
        } else {
            /* Make room for the new cell */
            for i in ((index+1)..(original_num_keys+1)).rev() {
                let destination = internal_node_cell(parent, i as usize) as *mut c_void;
                let source = internal_node_cell(parent, i as usize -1) as *const c_void;
                unsafe {libc::memcpy(destination, source, INTERNAL_NODE_CELL_SIZE);}
            }

            unsafe {
                *(internal_node_child(parent, index as usize)?) = child_page_num as u32;
                *internal_node_key(parent, index as usize) = child_max_key;
            }
        }   
        Ok(())
    }
}
