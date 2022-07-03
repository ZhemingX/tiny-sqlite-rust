pub mod meta_command;
pub mod prepare_statement;

use std::fmt;

use super::util::{zascii, str2dst};
use crate::db::tree::{ID_OFFSET, EMAIL_OFFSET, USERNAME_OFFSET, ID_SIZE, USERNAME_SIZE, EMAIL_SIZE};

use libc::{self, c_void, c_char, size_t};

#[derive(Debug)]
pub enum StatementType {
    StatementInsert,
    StatementSelect,
}

impl Default for StatementType {
    fn default() -> Self {
        Self::StatementInsert
    }
}

const COLUMN_USERNAME_SIZE: usize = 32;
const COLUMN_EMAIL_SIZE: usize = 255;

// hard code table row currently.
#[derive(Debug)]
pub struct Row {
    pub id: u32,
    pub email: [c_char; COLUMN_EMAIL_SIZE + 1],
    pub username: [c_char; COLUMN_USERNAME_SIZE + 1],
}

impl Default for Row {
    fn default() -> Self {
        Self {
            id: 0,
            email: [0 as c_char; COLUMN_EMAIL_SIZE + 1],
            username: [0 as c_char; COLUMN_USERNAME_SIZE + 1],
        }
    }
}

impl fmt::Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return unsafe {
            write!(
            f,
            "({},{},{})",
            self.id,
            zascii(&self.username),
            zascii(&self.email),
            )
        };
    }
}

impl Row {
    fn get_id_mut_ptr(&self) -> *mut u32 {
        unsafe {
            ((self as *const _) as *const u8)
                .offset(memoffset::offset_of!(Row, id) as isize)
                as *mut u32
        }
    }

    fn get_username_mut_ptr(&self) -> *mut c_char {
        unsafe {
            ((self as *const _) as *const u8)
                .offset(memoffset::offset_of!(Row, username) as isize)
                as *mut c_char
        }
    }

    fn get_email_mut_ptr(&self) -> *mut c_char {
        unsafe {
            ((self as *const _) as *const u8)
                .offset(memoffset::offset_of!(Row, email) as isize)
                as *mut c_char
        }
    }

    pub fn serialize_row(&self, dst: *mut c_void) {
        unsafe {
            libc::memcpy(
                (dst as *const u8)
                    .offset(ID_OFFSET as isize)
                    as *mut c_void,
                self.get_id_mut_ptr()
                    as *mut c_void,
                ID_SIZE as size_t
            );
            libc::memcpy(
                (dst as *const u8)
                    .offset(USERNAME_OFFSET as isize)
                    as *mut c_void,
                self.get_username_mut_ptr()
                    as *mut c_void,
                USERNAME_SIZE as size_t
            );
            libc::memcpy(
                (dst as *const u8)
                    .offset(EMAIL_OFFSET as isize)
                    as *mut c_void,
                self.get_email_mut_ptr()
                    as *mut c_void,
                EMAIL_SIZE as size_t
            );
        }
    }

    pub fn deserialize_row(&mut self, src: *const c_void) {
        unsafe {
            libc::memcpy(
                self.get_id_mut_ptr()
                    as *mut c_void,
                (src as *const u8)
                    .offset(ID_OFFSET as isize)
                    as *const c_void,
                ID_SIZE as size_t
            );
            libc::memcpy(
                self.get_username_mut_ptr()
                    as *mut c_void,
                (src as *const u8)
                    .offset(USERNAME_OFFSET as isize)
                    as *const c_void,
                USERNAME_SIZE as size_t
            );
            libc::memcpy(
                self.get_email_mut_ptr()
                    as *mut c_void,
                (src as *const u8)
                    .offset(EMAIL_OFFSET as isize)
                    as *const c_void,
                EMAIL_SIZE as size_t
            );
        }
    }

    pub fn set_username(&mut self, username: &str) {
            let r_username_ptr = self.get_username_mut_ptr();
            let _ = str2dst(r_username_ptr, username);
    }

    pub fn set_email(&mut self, email: &str) {
            let r_email_ptr = self.get_email_mut_ptr();
            let _ = str2dst(r_email_ptr, email);
    }
}
#[derive(Default, Debug)]
pub struct Statement {
    stmt_type: StatementType,
    row_to_insert: Row, // only insert by insert statement
}

impl Statement {
    pub fn new() -> Statement {
        Statement::default()
    }
}
