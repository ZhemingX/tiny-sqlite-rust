use std::path::{Path, PathBuf};
use std::mem;

mod cli;
mod db;
mod service;
#[macro_use]
mod util;

use cli::header::print_sqlite_logo;
use cli::run_loop;
use service::Row;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(short, long, parse(from_os_str), value_name = "FILE")]
    db: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    let mut db_name = "default.db";
    if let Some(_db_name) = cli.db.as_deref() {
        db_name = _db_name.to_str().unwrap();
        println!("DB file: {:?}\n", db_name);
    }

    print_sqlite_logo();

    run_loop(move |s| {
        println!("implement sql service! for {}", s);
    });
}
