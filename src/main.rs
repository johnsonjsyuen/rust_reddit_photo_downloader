use std::sync::Arc;
use anyhow::Error;
use async_channel::{Receiver, Sender};

use clap::Parser;
use reqwest::Client;
use tokio::sync::{Barrier, RwLock};

use tokio::task::JoinHandle;

use crate::download_file::download_a_file;
use crate::download_listing_page::{produce_links_from_page};

mod download_file;
mod download_listing_page;

#[derive(clap::ValueEnum, Clone, Debug)]
enum Period {
    All,
    Year,
    Month,
    Day,
}

impl Period {
    fn as_str(&self) -> &'static str {
        match self {
            Period::All => "all",
            Period::Year => "year",
            Period::Month => "month",
            Period::Day => "day"
        }
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value_t = 0)]
    max_pages: u8,
    #[clap(short, long, value_parser, default_value_t = 100)]
    concurrency: usize,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args: Args = Args::parse();
    let client = reqwest::Client::new();

    // Barrier needed to sync termination of program
    let completion_barrier = Arc::new(Barrier::new(args.concurrency));

    let (url_chan_send, url_chan_recv) = async_channel::unbounded::<String>();

    // Get and extract urls from Subreddit, send it to url channel
    let comp_barr_1 = completion_barrier.clone();
    let url_producer = produce_urls(
        client, url_chan_send,
        args.max_pages,
        comp_barr_1);


    // Download the images by consuming urls from channel
    let mut url_consumers = consume_urls(completion_barrier, url_chan_recv, args.concurrency);

    // Await everything
    url_consumers.insert(0,url_producer);
    futures::future::join_all(url_consumers).await;

    Ok(())
}

fn consume_urls(completion_barrier: Arc<Barrier>, url_chan_recv: Receiver<String>, concurrency: usize) -> Vec<JoinHandle<Result<(), Error>>> {
    let client = Client::new();
    (1..concurrency).map(
        |_| {
            let recv_clone = url_chan_recv.clone();
            let client2_clone = client.clone();
            let bz = completion_barrier.clone();
            tokio::spawn(async move {
                while let Ok(url) = recv_clone.recv().await {
                    download_a_file(&url, "./jsa/",client2_clone.clone()).await.unwrap();
                }
                bz.wait().await;
                Ok(())
            })
        }
    ).collect::<Vec<_>>()
}

fn produce_urls(client: Client,
                url_chan_send: Sender<String>,
                max_pages: u8,
                completion_barrier: Arc<Barrier>
                ) -> JoinHandle<Result<(), Error>> {
    let url_producer: JoinHandle<Result<(), Error>> = tokio::spawn(async move {
        for i in 1..1934{
            let url = format!("https://cloudfront.tubi.video/e5be87a5-c090-44f4-ab12-490957722f0a/pf1sf7a9/segment-{}.ts", i);
            url_chan_send.send(url.clone()).await?;
        };
        url_chan_send.close();
        completion_barrier.clone().wait().await;
        Ok(())
    });
    url_producer
}
