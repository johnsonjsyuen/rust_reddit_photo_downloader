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
    period: Period
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args:Args = Args::parse();
    let max_pages = 10;
    let mut pages_downloaded = 1;

    let mut after = String::new();
    while &pages_downloaded < &max_pages{
        match download_a_page(args.subreddit.clone(), &args.period.as_str(), &after).await?{
            None => break,
            Some(next) => after = next
        }
        };

    Ok(())
}
