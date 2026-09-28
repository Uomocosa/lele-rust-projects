pub mod error;
pub use error::Error;

mod brp;
mod cdp;
mod cli;
mod config;
mod process;
mod report;
mod web;

fn main() -> Result<(), Error> {
    cli::run()
}
