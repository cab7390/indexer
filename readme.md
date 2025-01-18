# Scraper and Indexer

A Rust-based tool for scraping tags and posts from an API, saving the data to JSON files, and building an optimized index for fast querying.

### Key Features

- **Failsafe Operation**: Automatically saves progress on interruption (e.g., `Ctrl+C` or terminal signal) and resumes seamlessly on the next run.
- **Efficient Indexing**: Uses `roaringbitmaps` for compact storage and fast set operations, enabling extremely low-latency queries.
- **Optimized Queries**: Improves query performance by sorting tags by frequency and leveraging efficient set intersections.

### Usage

Set up a `.env` file with `ENDPOINT`, `API_KEY`, and `USER_ID`, then run the tool:
```bash
cargo run --release
```

Scraped data will be saved to `tags.json`, `posts.json`, and `state.json`. The index enables rapid filtering of posts based on tags, even with millions of entries.