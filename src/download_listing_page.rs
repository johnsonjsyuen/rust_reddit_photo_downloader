use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct ListingDetail {
    title: String,
    id: String,
    url: String,
    is_video: bool,
    domain: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Listing {
    data: ListingDetail,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct ListingData {
    children: Vec<Listing>,
    after: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct ListingResponse {
    kind: String,
    data: ListingData,
}

async fn parse_links_from_page(
    listing_response: ListingResponse,
) -> Result<(Option<String>, Vec<String>), anyhow::Error> {
    let urls = listing_response
        .data
        .children
        .into_iter()
        .filter_map(|child| {
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
        })
        .collect();

    Ok((listing_response.data.after, urls))
}

pub async fn produce_links_from_page(
    subreddit: &str,
    period: &str,
    after_token: &str,
    client: Client,
) -> Result<(Option<String>, Vec<String>), anyhow::Error> {
    let url = format!(
        "https://www.reddit.com/r/{}/top.json?limit=100&sort=top&t={}&after={}",
        subreddit, period, after_token
    );
    let response = client
        .get(url)
        .send()
        .await?
        .json::<ListingResponse>()
        .await?;

    Ok(parse_links_from_page(response).await?)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use crate::download_listing_page::{
        parse_links_from_page, Listing, ListingData, ListingDetail, ListingResponse,
    };

    #[tokio::test]
    async fn able_to_load_test_file_from_resources() {
        let mut filepath = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        filepath.push("src/resources/listing.json");

        let contents = fs::read_to_string(filepath)
            .expect("Should have been able to read the test resource file");

        let deserialized: ListingResponse = serde_json::from_str(&contents).unwrap();

        let expected = ListingResponse {
            kind: "Listing".to_string(),
            data: ListingData {
                children: vec![Listing {
                    data: ListingDetail {
                        title: "Heat index was 110 degrees so we offered him a cold drink. \
                        He went for a full body soak instead"
                            .to_string(),
                        id: "90bu6w".to_string(),
                        url: "https://v.redd.it/gyh95hiqc0b11".to_string(),
                        is_video: true,
                        domain: "v.redd.it".to_string(),
                    },
                }],
                after: Some("t3_koo1z8".to_string()),
            },
        };
        assert_eq!(deserialized, expected);
    }

    #[tokio::test]
    async fn able_to_get_links() {
        let mut filepath = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        filepath.push("src/resources/listing.json");

        let contents = fs::read_to_string(filepath)
            .expect("Should have been able to read the test resource file");

        let deserialized: ListingResponse = serde_json::from_str(&contents).unwrap();

        let (token, links) = parse_links_from_page(deserialized).await.unwrap();
        let expected_token = "t3_koo1z8";
        let expected_links = vec![""];
        assert_eq!(expected_token, token.unwrap());
        assert_eq!(expected_links, links);
    }
}
