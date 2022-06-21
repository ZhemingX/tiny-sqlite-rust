pub enum MetaCommandResult {
    MetaCmdSuccess,
    MetaCmdExit,
    MetaCmdUnrecognizedCmd,
}

pub fn do_meta_command(cmd: &str) -> MetaCommandResult {
    match cmd {
        ".exit;" => MetaCommandResult::MetaCmdExit,
        ".btree;" => {
            // TODO
            println!("print btree\n");
            MetaCommandResult::MetaCmdSuccess
        },
        ".constants;" => {
            // TODO
            println!("print constants\n");
            MetaCommandResult::MetaCmdSuccess
        } 
        _ => {
            // unrecognized command
            MetaCommandResult::MetaCmdUnrecognizedCmd
        }
    }
}


