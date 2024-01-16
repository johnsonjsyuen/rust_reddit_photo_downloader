use anyhow::Error;
use async_channel::{Receiver, Sender};
use std::sync::Arc;

use clap::Parser;
use reqwest::Client;
use tokio::sync::{Barrier, RwLock};

use tokio::task::JoinHandle;

use rusqlite::{params, Connection, Result};

use crate::download_file::download_a_file;
use crate::download_listing_page::produce_links_from_page;

mod download_file;
mod download_listing_page;
mod models;

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
            Period::Day => "day",
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
    concurrency: usize,
}

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
const DOWNLOAD_DIRECTORY: &str = "./pics";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    init_db(None);

    let args: Args = Args::parse();
    let client = Client::builder().user_agent(USER_AGENT).build()?;
    let subreddit = Arc::new(RwLock::new(args.subreddit));

    // Barrier needed to sync termination of program
    let completion_barrier = Arc::new(Barrier::new(args.concurrency));

    let (url_chan_send, url_chan_recv) = async_channel::unbounded::<String>();

    // Get and extract urls from Subreddit, send it to url channel
    let comp_barr_1 = completion_barrier.clone();
    let subreddit_1 = Arc::clone(&subreddit);
    let url_producer = produce_urls(
        client,
        url_chan_send,
        args.max_pages,
        args.period.as_str().to_owned(),
        comp_barr_1,
        subreddit_1,
    );

    // Download the images by consuming urls from channel
    let mut url_consumers = consume_urls(
        &subreddit,
        completion_barrier,
        url_chan_recv,
        args.concurrency,
    );

    // Await everything
    url_consumers.insert(0, url_producer);
    futures::future::join_all(url_consumers).await;

    Ok(())
}

fn init_db(db_name: Option<String>) -> Result<()> {
    let path = db_name.unwrap_or_else(||"reddit_listings.db".to_string());
    let db = Connection::open(path)?;
    db.execute(
        "CREATE TABLE IF NOT EXISTS listing (
            id       TEXT PRIMARY KEY,
            title    TEXT NOT NULL,
            url      TEXT NOT NULL,
            is_video INTEGER NOT NULL,
            domain   TEXT BLOB
        ) STRICT",
        (), // empty list of parameters.
    )?;
    Ok(())
}

fn consume_urls(
    subreddit: &Arc<RwLock<String>>,
    completion_barrier: Arc<Barrier>,
    url_chan_recv: Receiver<String>,
    concurrency: usize,
) -> Vec<JoinHandle<Result<(), Error>>> {
    let client = Client::new();
    (1..concurrency)
        .map(|_| {
            let recv_clone = url_chan_recv.clone();
            let arc_subreddit = Arc::clone(subreddit);
            let client2_clone = client.clone();
            let arc_barrier = completion_barrier.clone();
            tokio::spawn(async move {
                let subreddit_clone = arc_subreddit.read().await;
                while let Ok(url) = recv_clone.recv().await {
                    download_a_file(
                        &url,
                        &format!("{}/{}/", DOWNLOAD_DIRECTORY, subreddit_clone.clone()),
                        client2_clone.clone(),
                    )
                    .await
                    .unwrap();
                }
                arc_barrier.wait().await;
                Ok(())
            })
        })
        .collect::<Vec<_>>()
}

fn produce_urls(
    client: Client,
    url_chan_send: Sender<String>,
    max_pages: u8,
    period: String,
    completion_barrier: Arc<Barrier>,
    subreddit_arc: Arc<RwLock<String>>,
) -> JoinHandle<Result<(), Error>> {
    let url_producer: JoinHandle<Result<(), Error>> = tokio::spawn(async move {
        let mut after = String::new();
        let mut count = 0;
        let mut pages_downloaded = 0;
        let subreddit = subreddit_arc.read().await;
        while max_pages == 0_u8 || pages_downloaded < max_pages {
            let produce =
                produce_links_from_page(&subreddit, &period, &after, client.clone()).await?;
            for url in produce.1.into_iter() {
                count += 1;
                url_chan_send.send(url.clone()).await?;
            }
            match produce.0 {
                None => {
                    break;
                }
                Some(next) => {
                    pages_downloaded += 1;
                    after = next
                }
            }
        }
        println!("Sent {} urls to download from {}", count, &subreddit);
        url_chan_send.close();
        completion_barrier.clone().wait().await;
        Ok(())
    });
    url_producer
}
