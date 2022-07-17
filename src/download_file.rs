use std::fs;
use std::fs::File;
use std::io::Write;

use anyhow::Result;

pub(crate) async fn download_a_file(url: &str, dest_dir: &str) -> Result<(), anyhow::Error> {
    let response = reqwest::get(url).await?;
    fs::create_dir_all(dest_dir);

    let mut dest = {
        let fname = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("tmp.bin");

        let fname = format!("{}{}", dest_dir, fname);

        println!("file to download: '{}'", fname);
        File::create(fname)?
    };
    let mut content = response.bytes().await?;
    dest.write_all(&content).expect("Failed to write a file, permission issue?");
    Ok(())
}