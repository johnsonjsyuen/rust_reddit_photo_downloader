mod download_file;

use std::io::{copy, Write};
use std::fs::File;
use download_file::download_a_file;
use anyhow::Result;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct ListingDetail {
    title: String,
    id: String,
    url: String,
    is_video: bool,
    domain: String
}

#[derive(Serialize, Deserialize, Debug)]
struct Listing {
    data: ListingDetail,
}

#[derive(Serialize, Deserialize, Debug)]
struct ListingData {
    children: Vec<Listing>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ListingResponse {
    kind: String,
    data: ListingData
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let target = "https://i.redd.it/403jehnfqzb91.jpg";
    download_a_file(target).await?;
    let response = reqwest::get("https://www.reddit.com/r/aww.json").await?
        .json::<ListingResponse>()
        .await?;

    println!("listing: {:?}", response);

    Ok(())
}
