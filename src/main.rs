use std::path::PathBuf;

mod cli;
mod db;
mod service;
#[macro_use]
mod util;

use cli::header::print_sqlite_logo;
use cli::run_loop;
use service::Row;
use crate::util::{str2dst, zascii};

use clap::Parser;
use libc::{self, c_char, c_void};
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

    // let mut r = Row::default();
    // r.id = 3;
    // r.set_email("aaaaa.com");
    // r.set_username("dddddxzzzzm");
    // println!("email: {}", zascii(&r.email));
    // println!("username: {}", zascii(&r.username));
    // println!("Row: {}", r);

    // let page: *mut c_void = unsafe {
    //     // Cache miss. Allocate memory and load from file.
    //     libc::malloc(1000 as usize) as *mut c_void
    // };

    // r.serialize_row(page);

    // let mut r2 = Row::default();
    // r2.deserialize_row(page as *const c_void);

    // println!("r2: {}", r2);

    // unsafe{libc::free(page);}

    print_sqlite_logo();

    run_loop(move |s| {
        println!("implement sql service! for {}", s);
    });
}
