use std::fmt;

use crate::service::{Statement, StatementType};
use crate::service::{COLUMN_EMAIL_SIZE, COLUMN_USERNAME_SIZE};

#[derive(Debug)]
pub enum PrepareResult {
    PrepareSuccess,
    PrepareNegativeId,
    PrepareStringTooLong,
    PrepareSyntaxError,
    PrepareUnrecognizeStmt(String),
}

impl fmt::Display for PrepareResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrepareResult::PrepareSuccess => write!(f, "Successfully prepared!"),
            PrepareResult::PrepareNegativeId => write!(f, "ID must be positive."),
            PrepareResult::PrepareStringTooLong => write!(f, "String is too long."),
            PrepareResult::PrepareSyntaxError => {
                write!(f, "Syntax error. Could not parse statement.")
            }
            PrepareResult::PrepareUnrecognizeStmt(stmt) => {
                write!(f, "Unrecognized keyword at start of \'{}\'.", stmt)
            }
        }
    }
}

pub struct PrepareService {}

impl PrepareService {
    pub fn new() -> PrepareService {
        Self {}
    }

    pub fn prepare_statement(&self, line: &str, stmt: &mut Statement) -> PrepareResult {
        if line.starts_with("insert") {
            return self.prepare_insert(line, stmt);
        }

        if line.starts_with("select") {
            stmt.stmt_type = StatementType::StatementSelect;
            return PrepareResult::PrepareSuccess;
        }

        return PrepareResult::PrepareUnrecognizeStmt(line.to_string());
    }

    fn prepare_insert(&self, line: &str, stmt: &mut Statement) -> PrepareResult {
        stmt.stmt_type = StatementType::StatementInsert;

        let line_partition: Vec<&str> = line.split(' ').collect();

        if line_partition.len() != 4 {
            return PrepareResult::PrepareSyntaxError;
        }

        let id = match line_partition[1].parse::<i32>() {
            Ok(id_num) => {
                if id_num < 0 {
                    return PrepareResult::PrepareNegativeId;
                }
                id_num as u32
            }
            Err(_) => {
                return PrepareResult::PrepareSyntaxError;
            }
        };

        let username = if line_partition[2].len() <= COLUMN_USERNAME_SIZE {
            line_partition[2]
        } else {
            return PrepareResult::PrepareStringTooLong;
        };

        let email_len = line_partition[3].len() - 1;
        let email = if line_partition[3].len() <= COLUMN_EMAIL_SIZE {
            &line_partition[3][..email_len]
        } else {
            return PrepareResult::PrepareStringTooLong;
        };

        stmt.row_to_insert.id = id;
        stmt.row_to_insert.set_username(username);
        stmt.row_to_insert.set_email(email);

        PrepareResult::PrepareSuccess
    }
}
