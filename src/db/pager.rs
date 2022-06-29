use std::ffi::CString;
use std::ptr;

use crate::db::error::DbError;

use libc::{c_char, c_int, c_void};

pub const PAGE_SIZE: usize = 4096;
const TABLE_MAX_PAGES: usize = 100;

pub type DbResult<T> = Result<T, DbError>;

pub struct Pager {
    file_descripter: c_int,
    file_length: usize,
    pub num_pages: usize,
    pages: [*mut c_void; TABLE_MAX_PAGES as usize],
}

impl Default for Pager {
    fn default() -> Self {
        Self {
            file_descripter: 0,
            file_length: 0,
            num_pages: 0,
            pages: [ptr::null_mut::<c_void>(); TABLE_MAX_PAGES],
        }
    }
}

impl Pager {
    pub fn new(filename: &str) -> DbResult<Self> {
        let c_filename = CString::new(filename).unwrap().as_ptr() as *const c_char;

        let fd = unsafe {
            libc::open(
                c_filename,
                libc::O_RDWR | libc::O_CREAT | libc::S_IWUSR as i32 | libc::S_IRUSR as i32,
            )
        };

        if fd == -1 {
            return Err(DbError::Other("Unable to open file".to_string()));
        }

        let file_length = unsafe { libc::lseek(fd, 0, libc::SEEK_END) };

        if file_length as usize % PAGE_SIZE != 0 {
            return Err(DbError::Other(
                "Db file is not a whole number of pages. Corrupt file.".to_string(),
            ));
        }

        let pager = Pager {
            file_descripter: fd,
            file_length: file_length as usize,
            num_pages: file_length as usize / PAGE_SIZE,
            pages: [ptr::null_mut::<c_void>(); TABLE_MAX_PAGES],
        };

        Ok(pager)
    }

    pub fn get_page(&mut self, page_num: usize) -> DbResult<*mut c_void> {
        if page_num > TABLE_MAX_PAGES {
            return Err(DbError::Other(format!(
                "Tried to fetch page number out of bounds. {} > {}",
                page_num, TABLE_MAX_PAGES
            )));
        }

        if self.pages[page_num as usize].is_null() {
            let page: *mut c_void = unsafe {
                // Cache miss. Allocate memory and load from file.
                libc::malloc(PAGE_SIZE as usize) as *mut c_void
            };
            let mut num_pages = self.file_length / PAGE_SIZE;

            // We might save a partial page at the end of the file
            if self.file_length % PAGE_SIZE != 0 {
                num_pages += 1;
            }

            if page_num <= num_pages {
                unsafe {
                    libc::lseek(
                        self.file_descripter,
                        (page_num * PAGE_SIZE) as i64,
                        libc::SEEK_SET,
                    );
                    let bytes_read: libc::ssize_t =
                        libc::read(self.file_descripter, page, PAGE_SIZE as usize);

                    if bytes_read == -1 {
                        return Err(DbError::Other("Error reading file".to_owned()));
                    }
                }
            }

            self.pages[page_num] = page;

            if page_num >= self.num_pages {
                self.num_pages += 1;
            }
        }

        Ok(self.pages[page_num])
    }

    pub fn pager_flush(&mut self, page_num: usize) -> DbResult<()> {
        if self.pages[page_num].is_null() {
            return Err(DbError::Other("Tried to flush null page".to_owned()));
        }

        let offset = unsafe {
            libc::lseek(
                self.file_descripter,
                (page_num * PAGE_SIZE) as i64,
                libc::SEEK_SET,
            )
        };

        if offset == -1 {
            return Err(DbError::Other("Error seeking".to_owned()));
        }

        let bytes_written: libc::ssize_t =
            unsafe { libc::write(self.file_descripter, self.pages[page_num], PAGE_SIZE) };

        if bytes_written == -1 {
            return Err(DbError::Other("Error writing".to_owned()));
        }

        Ok(())
    }
}
