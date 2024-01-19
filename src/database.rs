use anyhow::Error;
use duckdb::{Connection, params};
use crate::models::ListingResponse;

pub async fn init_db(db_path: String) -> duckdb::Result<()> {
    let db = Connection::open(db_path)?;
    db.execute_batch(
        r"CREATE SEQUENCE seq;
        CREATE TABLE IF NOT EXISTS listing (
            id        VARCHAR PRIMARY KEY,
            subreddit VARCHAR NOT NULL,
            title     VARCHAR NOT NULL,
            url       VARCHAR NOT NULL,
            is_video  BOOLEAN NOT NULL,
            domain    VARCHAR
        );"
    )?;
    Ok(())
}

pub async fn export_db(db_path: &str) -> duckdb::Result<()> {
    let db = Connection::open(db_path)?;
    db.execute_batch(
        r"COPY listing TO 'output.parquet' (FORMAT PARQUET);"
    )?;
    Ok(())
}

pub fn store_listings_in_db(db_path: &str, subreddit: &str, listing_response: ListingResponse) -> duckdb::Result<(), Error> {
    let db = Connection::open(db_path)?;
    let mut app = db.appender("listing")?;
    let listings = listing_response.data.children;
    for listing in listings {
        app.append_row(params![
            listing.data.id,
            subreddit,
            listing.data.title,
            listing.data.url,
            listing.data.is_video,
            listing.data.domain
            ])?;
    }
    Ok(())
}
