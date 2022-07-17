use crate::download_listing_page::download_a_page;

mod download_file;
mod download_listing_page;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{hint, thread};

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[clap(short, long, value_parser)]
    name: String,

    /// Number of times to greet
    #[clap(short, long, value_parser, default_value_t = 1)]
    count: u8,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let max_pages = 10;
    let mut pages_downloaded = 1;

    let mut after = String::new();
    while &pages_downloaded < &max_pages{
        match download_a_page("aww".to_owned(), "year", &after).await?{
            None => break,
            Some(next) => after = next
        }
        };

    Ok(())
}
