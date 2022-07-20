use std::fs;
use std::fs::File;
use std::io::Write;

use anyhow::Result;
use reqwest::Client;

pub(crate) async fn download_a_file(url: &str, dest_dir: &str, client: Client) -> Result<(), anyhow::Error> {
    let response = client.get(url).send().await?;
    fs::create_dir_all(dest_dir)?;

    let mut dest = {
        let fname = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("tmp.bin");

        let fname = format!("{}{}", dest_dir, fname);

        //println!("file to download: '{}'", fname);
        File::create(fname)?
    };
    let content = response.bytes().await?;
    dest.write_all(&content).expect("Failed to write a file, permission issue?");
    Ok(())
}