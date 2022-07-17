use std::fs::File;

use anyhow::Result;
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};

use crate::download_file::download_a_file;

#[derive(Serialize, Deserialize, Debug)]
struct ListingDetail {
    title: String,
    id: String,
    url: String,
    is_video: bool,
    domain: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Listing {
    data: ListingDetail,
}

#[derive(Serialize, Deserialize, Debug)]
struct ListingData {
    children: Vec<Listing>,
    after: String
}

#[derive(Serialize, Deserialize, Debug)]
struct ListingResponse {
    kind: String,
    data: ListingData,
}

pub async fn download_a_page(subreddit: &str, period: &str, after_token: &str) -> Result<(), anyhow::Error> {
    let url = format!("https://www.reddit.com/r/{}/top.json?limit=100&sort=top&t={}&after={}", subreddit, period, after_token);
    let response = reqwest::get(url).await?
        .json::<ListingResponse>()
        .await?;

    let join_handles = response.data.children.into_iter().map(|child| {
        tokio::spawn(async move {
            if !child.data.is_video {
                let url = &child.data.url;
                println!("URL:{}", &url);
                if url.ends_with(".jpg") || url.ends_with(".png") {
                    download_a_file(&url, "./pics/").await.unwrap();
                }
            }
        })
    }).collect::<Vec<_>>();

    try_join_all(join_handles).await.unwrap();
    Ok(())
}