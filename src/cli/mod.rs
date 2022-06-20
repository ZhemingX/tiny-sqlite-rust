pub mod header;

use rustyline::error::ReadlineError;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::Editor;
use rustyline_derive::{Completer, Helper, Highlighter, Hinter};

#[derive(Completer, Helper, Highlighter, Hinter)]
pub struct EditHelper {
    _match_script_end_validator: (),
    _highlighter: (),
}

impl EditHelper {
    pub fn new() -> Self {
        Self {
            _match_script_end_validator: (),
            _highlighter: (),
        }
    }
}

impl Validator for EditHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        let i = ctx.input();
        if i.ends_with(";") {
            Ok(ValidationResult::Valid(None))
        } else {
            Ok(ValidationResult::Incomplete)
        }
    }
}

pub fn run_loop<F>(func: F)
where
    F: Fn(&str),
{
    let mut rl = Editor::new();
    let edit_helper = EditHelper::new();
    rl.set_helper(Some(edit_helper));

    let prompt = "sqlite >> ";
    loop {
        let readline = rl.readline(prompt);
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                func(&line)
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
