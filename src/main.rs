mod download_file;

use std::io::{copy, Write};
use std::fs::File;
use download_file::download_a_file;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let target = "https://i.redd.it/403jehnfqzb91.jpg";
    download_a_file(target).await?;
    Ok(())
}
