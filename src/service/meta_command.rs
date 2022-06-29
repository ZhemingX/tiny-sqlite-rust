pub enum MetaCommandResult {
    MetaCmdSuccess,
    MetaCmdExit,
    MetaCmdUnrecognizedCmd,
}

pub struct MetaCommandService {}

impl MetaCommandService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn do_meta_command(&self, cmd: &str) -> MetaCommandResult {
        match cmd {
            ".exit;" => MetaCommandResult::MetaCmdExit,
            ".btree;" => {
                // TODO
                println!("print btree\n");
                MetaCommandResult::MetaCmdSuccess
            }
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
}
