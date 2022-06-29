pub mod meta_command;
pub mod prepare_statement;

use std::fmt;

use super::util::arr2string;

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
    pub email: [char; COLUMN_EMAIL_SIZE + 1],
    pub username: [char; COLUMN_USERNAME_SIZE + 1],
}

impl Default for Row {
    fn default() -> Self {
        Self {
            id: 0,
            email: ['0'; COLUMN_EMAIL_SIZE + 1],
            username: ['0'; COLUMN_USERNAME_SIZE + 1],
        }
    }
}

impl fmt::Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(
            f,
            "({},{},{})",
            self.id,
            arr2string(&self.username),
            arr2string(&self.email)
        );
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
