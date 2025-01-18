use std::{fs::File, io::BufWriter};

use indexer::{
    api::client::ApiClient,
    index::Index,
    scraper::{post_scraper::PostScraper, state_manager::StateManager, tag_scraper::TagScraper},
};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT_LANGUAGE, USER_AGENT};
use tracing::info;

fn init_tracing() {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
}

/// Create a reqwest client with the necessary headers
fn create_client() -> reqwest::Client {
    let mut headers = HeaderMap::default();
    headers.insert(USER_AGENT, HeaderValue::from_str("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36").unwrap());
    headers.insert(
        ACCEPT_LANGUAGE,
        HeaderValue::from_str("en-US,en;q=0.9").unwrap(),
    );

    reqwest::Client::builder()
        .brotli(true)
        .gzip(true)
        .deflate(true)
        .default_headers(headers)
        .build()
        .unwrap()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();
    dotenvy::dotenv().expect("Failed to load .env file");

    let endpoint = dotenvy::var("ENDPOINT").expect("ENDPOINT must be set");
    let api_key = dotenvy::var("API_KEY").expect("API_KEY must be set");
    let user_id = dotenvy::var("USER_ID").expect("USER_ID must be set");

    let api_client = ApiClient::builder()
        .client(create_client())
        .endpoint(endpoint)
        .api_key(api_key)
        .user_id(user_id)
        .build();

    // Listen for ctrl-c
    let ctrl_c_task = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for ctrl-c");
    };

    // Scraped tags will be written to this file
    let tag_output = BufWriter::new(
        File::options()
            .append(true)
            .create(true)
            .open("tags.json")
            .expect("Failed to open tags.json"),
    );

    // Scraped posts will be written to this file
    let post_output = BufWriter::new(
        File::options()
            .append(true)
            .create(true)
            .open("posts.json")
            .expect("Failed to open posts.json"),
    );

    let state_manager = StateManager::new("state.json").expect("Failed to load state file");
    let tag_scraper = TagScraper::new(tag_output, state_manager.clone(), api_client.clone());
    let post_scraper = PostScraper::new(post_output, state_manager.clone(), api_client.clone());

    let tag_scraper_task = async move {
        tag_scraper.run().await.unwrap();
    };

    let post_scraper_task = async move {
        post_scraper.run().await.unwrap();
    };

    tokio::select! {
        _ = post_scraper_task => {
            info!("Finished Scraping Posts");
            state_manager.save_state("state.json").await?;
        }
        _ = tag_scraper_task => {
            info!("Finished Scraping Tags");
            state_manager.save_state("state.json").await?;
        }
        _ = ctrl_c_task => {
            info!("Saving State");
            state_manager.save_state("state.json").await?;
        }
    }

    Ok(())
}

/// Example of building an index from the scraped data and querying it
///
/// From my benchmarks, most queries take less than 2ms to complete with an index of around 10 million posts
fn _build_index() {
    let index = Index::generate("posts.json", "tags.json").expect("Failed to generate index");

    let query = vec![String::from("cat"), String::from("dog")];

    let start = std::time::Instant::now();
    index.get_images_all_tags_lazy(query).unwrap().count();
    let duration = start.elapsed();
    println!("Query took: {:?}", duration);
}
