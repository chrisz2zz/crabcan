use std::process::exit;

use errors::exit_with_retcode;

#[macro_use]
extern crate scan_fmt;

mod capabilities;
mod child;
mod cli;
mod config;
mod container;
mod errors;
mod hostname;
mod ipc;
mod mounts;
mod namespace;
mod resources;
mod syscalls;

fn main() {
    match cli::parse_args() {
        Ok(args) => {
            log::info!("{args:?}");
            exit_with_retcode(container::start(args));
        }
        Err(e) => {
            log::error!("Error while parsing arguments:\n\t{}", e);
            exit(e.get_retcode());
        }
    }
}
