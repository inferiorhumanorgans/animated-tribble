pub mod globals;
pub mod vault;

// Subcommands
pub mod env;
pub mod rest;

#[macro_use] extern crate clap;
use clap::{App};

extern crate futures;

#[macro_use] extern crate hyper;
extern crate hyper_tls;

extern crate tokio_core;

extern crate serde;
extern crate serde_json;
extern crate shell_escape;

fn main() {
    let matches = App::new(globals::APP_NAME)
                           .version(globals::APP_VERSION)
                           .subcommand(env::init_subcommand())
                           .subcommand(rest::init_subcommand())
                           .get_matches();

    match matches.subcommand() {
        ("env",  Some(matches)) => env::do_subcommand(matches),
        ("rest", Some(matches)) => rest::do_subcommand(matches),
        ("",     None)          => println!("No subcommand was used"),
        _                       => unreachable!(),
    }
}
