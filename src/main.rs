use std::{hint, thread};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use clap::Parser;
use futures::future::try_join_all;
use tokio::sync::Barrier;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::download_file::download_a_file;
use crate::download_listing_page::{download_a_page, produce_links_from_page};

mod download_file;
mod download_listing_page;

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
    #[clap(short, long, value_parser, default_value_t = 10)]
    concurrency: usize,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args: Args = Args::parse();
    let subreddit = args.subreddit.clone();
    let mut pages_downloaded = 0;

    let client = reqwest::Client::new();
    let barrier = Arc::new(Barrier::new(args.concurrency));

    let (url_chan_send, url_chan_recv) = async_channel::unbounded::<String>();

    // Get the pages
    let bb = barrier.clone();
    let url_producer: JoinHandle<Result<(), anyhow::Error>> = tokio::spawn(async move {
        let mut after = String::new();
        let mut count = 0;
        while &args.max_pages == &0_u8 || &pages_downloaded < &args.max_pages {
            let produce = produce_links_from_page(subreddit.clone(), &args.period.as_str(), &after, client.clone()).await?;
            for url in produce.1.into_iter() {
                count += 1;
                url_chan_send.send(url.clone()).await?;
            };
            match produce.0 {
                None => {
                    break;
                }
                Some(next) => {
                    pages_downloaded += 1;
                    after = next
                }
            }
        };
        println!("Sent {} urls", count);
        url_chan_send.close();
        bb.clone().wait().await;
        Ok(())
    });

    // Download the images
    let client2 = reqwest::Client::new();
    let mut url_consumers: Vec<JoinHandle<Result<(), anyhow::Error>>> = (1..args.concurrency).map(
        |_| {
            let recv_clone = url_chan_recv.clone();
            let subreddit_clone = args.subreddit.clone();
            let client2_clone = client2.clone();
            let bz = barrier.clone();
            tokio::spawn(async move {
                while let Ok(url) = recv_clone.recv().await {
                    download_a_file(&url, &format!("./pics/{}/", subreddit_clone.clone()), client2_clone.clone()).await.unwrap();
                }
                bz.wait().await;
                Ok(())
            })
        }
    ).collect::<Vec<_>>();

    // Await everything
    url_consumers.insert(0,url_producer);
    futures::future::join_all(url_consumers).await;

    Ok(())
}
