use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

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


pub async fn produce_links_from_page(subreddit: &str, period: &str, after_token: &str, client: Client) -> Result<(Option<String>, Vec<String>), anyhow::Error> {
    let url = format!("https://www.reddit.com/r/{}/top.json?limit=100&sort=top&t={}&after={}", subreddit, period, after_token);
    let response = client.get(url).send().await?
        .json::<ListingResponse>()
        .await?;

    let urls = response.data.children.into_iter().filter_map(|child| {
        let url = child.data.url;
        //println!("URL:{}", &url);
        if !child.data.is_video {
            if url.ends_with(".jpg") || url.ends_with(".png") {
                Some(url)
            } else if child.data.domain == "i.imgur.com" {
                if url.ends_with(".gifv") {
                    Some(url.replace(".gifv", ".mp4"))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }).collect();

    Ok((response.data.after, urls))
}