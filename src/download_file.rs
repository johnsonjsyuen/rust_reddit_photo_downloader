use std::io::{copy, Write};
use std::fs::File;
use anyhow::Result;

pub(crate) async fn download_a_file(url: &str) ->Result<(), anyhow::Error>{
    let response = reqwest::get(url).await?;

    let mut dest = {
        let fname = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("tmp.bin");

        println!("file to download: '{}'", fname);
        println!("will be located under: '{:?}'", fname);
        File::create(fname)?
    };
    let mut content =  response.bytes().await?;
    dest.write_all(&content).expect("Failed to write a file, permission issue?");
    Ok(())
}