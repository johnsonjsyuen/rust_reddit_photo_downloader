use crate::download_listing_page::{download_a_page, produce_links_from_page};

mod download_file;
mod download_listing_page;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{hint, thread};
use async_channel::Receiver;

use clap::Parser;
use futures::future::try_join_all;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Receiver as broadcast_recv;
use tokio::task::JoinHandle;
use crate::download_file::download_a_file;

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
    #[clap(short, long, value_parser, default_value_t = 100)]
    concurrency: u8,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args:Args = Args::parse();
    let subreddit = args.subreddit.clone();
    let mut pages_downloaded = 0;
    let client = reqwest::Client::new();

    let (url_chan_send, url_chan_recv) = async_channel::unbounded::<String>();
    let (termination_channel_send, mut termination_channel_recv) = broadcast::channel::<String>(1);

    let mut subscriptions:Vec<broadcast_recv<String>> = vec!();

        for _ in (1..args.concurrency){
            subscriptions.push(termination_channel_send.subscribe());
        };

    // Get the pages
    let url_producer: JoinHandle<Result<(),anyhow::Error>> = tokio::spawn(async move {
    let mut after = String::new();
    while &args.max_pages == &0_u8 || &pages_downloaded < &args.max_pages{
        let produce = produce_links_from_page(subreddit.clone(), &args.period.as_str(), &after, client.clone()).await?;
        for url in produce.1.into_iter() {
            url_chan_send.send(url.clone()).await?;
        };
        match produce.0{
            None => {
                termination_channel_send.clone().send(String::new())?;
                break;
            },
            Some(next) => {
                pages_downloaded += 1;
                after = next
            }
        }
    };
        Ok(())
    });

    // Download the images
    let client2 = reqwest::Client::new();
    let mut url_consumers = subscriptions.into_iter().map(
        |mut term_sub|{ let recv_clone = url_chan_recv.clone();
            let subreddit_clone = args.subreddit.clone();
            let client2_clone = client2.clone();
            tokio::spawn(async move {
                loop  {
                    if let Ok(_) = term_sub.recv().await{
                        break
                    }
                    if let Ok(url) = recv_clone.recv().await{
                        download_a_file(&url, &format!("./pics/{}/", subreddit_clone.clone()), client2_clone.clone()).await.unwrap();
                    }
                }
            Ok(())
            })
        }
    ).collect::<Vec<_>>();

    // Await everything
    url_consumers.insert(0,url_producer);
    try_join_all(url_consumers);

    Ok(())
}
