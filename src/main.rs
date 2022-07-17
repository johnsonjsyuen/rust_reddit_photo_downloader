use crate::download_listing_page::download_a_page;

mod download_file;
mod download_listing_page;


#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    download_a_page("aww", "year", "").await?;
    Ok(())
}
