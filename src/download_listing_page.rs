use std::fs::File;

use anyhow::Result;
use futures::future::try_join_all;
use reqwest::Client;
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
    after: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ListingResponse {
    kind: String,
    data: ListingData,
}

pub async fn download_a_page(subreddit: String, period: &str, after_token: &str, client: reqwest::Client) -> Result<Option<String>, anyhow::Error> {
    let url = format!("https://www.reddit.com/r/{}/top.json?limit=100&sort=top&t={}&after={}", subreddit, period, after_token);
    let response = reqwest::get(url).await?
        .json::<ListingResponse>()
        .await?;

    let join_handles = response.data.children.into_iter().map(|child| {
        let subreddit_clone = subreddit.clone();
        let client_clone = client.clone();
        tokio::spawn(async move {
            let url = &child.data.url;
            println!("URL:{}", &url);
            if !child.data.is_video {
                if url.ends_with(".jpg") || url.ends_with(".png") {
                    download_a_file(&url, &format!("./pics/{}/", subreddit_clone), client_clone).await.unwrap();
                } else if child.data.domain == "i.imgur.com" {
                    if url.ends_with(".gifv") {
                        download_a_file(&url.replace(".gifv", ".mp4"),
                                        &format!("./pics/{}/", subreddit_clone),
                                        client_clone,
                        ).await.unwrap();
                    }
                }
            }
        })
    }).collect::<Vec<_>>();

    try_join_all(join_handles).await.unwrap();
    Ok(response.data.after)
}

pub async fn produce_links_from_page(subreddit: &str, period: &str, after_token: &str, client: Client) -> Result<(Option<String>,Vec<String>), anyhow::Error> {
    let url = format!("https://www.reddit.com/r/{}/top.json?limit=100&sort=top&t={}&after={}", subreddit, period, after_token);
    let response = reqwest::get(url).await?
        .json::<ListingResponse>()
        .await?;

    let urls = response.data.children.into_iter().map(|child| {
            let url = child.data.url;
            //println!("URL:{}", &url);
            if !child.data.is_video {
                if url.ends_with(".jpg") || url.ends_with(".png") {
                    Some(url)
                } else if child.data.domain == "i.imgur.com" {
                    if url.ends_with(".gifv") {
                        Some(url.replace(".gifv", ".mp4"))
                    }else{
                        None
                    }
                }else {
                    None
                }
            }else{
                None
            }
    }).flatten().collect::<Vec<String>>();

    Ok((response.data.after,urls))
}