use crate::download_listing_page::download_a_page;

mod download_file;
mod download_listing_page;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{hint, thread};

use clap::Parser;

#[derive(clap::ValueEnum, Clone, Debug)]
enum Period {
    Year,
    Month,
    Day,
}

impl Period {
    fn as_str(&self) -> &'static str {
        match self {
            Period::Year => "year",
            Period::Month => "month",
            Period::Day => "day"
        }
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser)]
    subreddit: String,
    #[clap(short, long, value_parser)]
    period: Period,
    #[clap(short, long, value_parser, default_value_t = 0)]
    max_pages: u8,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args:Args = Args::parse();
    let mut pages_downloaded = 0;

    let mut after = String::new();
    while &args.max_pages == &0_u8 ||&pages_downloaded < &args.max_pages{
        match download_a_page(args.subreddit.clone(), &args.period.as_str(), &after).await?{
            None => break,
            Some(next) => after = next
        }
        };

    Ok(())
}
