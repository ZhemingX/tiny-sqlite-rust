pub mod meta_command;
pub mod prepare_statement;

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

const COLUMN_USERNAME_SIZE: u32 = 32;
const COLUMN_EMAIL_SIZE: u32 = 255;

// hard code table row currently.
#[derive(Default, Debug)]
pub struct Row {
    id: u32,
    email: String,
    username: String,
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