# Reddit Picture Downloader

Download pictures from a Subreddit, 
sorted by top votes of a time period. 
Max pages can be specified or left as 0 to download until no listings remain.

Usage
```
./downloader --subreddit aww --period month  --max-pages 3
```

## Database export
Stores listings downloaded into a duckdb database, as well as in Parquet format.
Download does not fail if either operation fails.

## Compile in Docker for Linux
```
docker run --rm --user "$(id -u)":"$(id -g)" -v "$PWD":/usr/src/myapp -w /usr/src/myapp rust:1.68.2 cargo build --release
```
