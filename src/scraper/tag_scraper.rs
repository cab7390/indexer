use std::{io::Write, num::NonZeroU32, sync::Arc};

use futures::StreamExt;
use governor::{Quota, RateLimiter};
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::{api::client::ApiClient, models::Tag, scraper::state_manager::ScrapeError};

use super::state_manager::StateManager;



pub struct TagScraper<W: Write> {
    state_manager: StateManager,
    client: ApiClient,
    output: Arc<Mutex<W>>,
    requests_per_second: NonZeroU32,
}

impl<W: Write> TagScraper<W> {
    pub fn new(output: W, state_manager: StateManager, client: ApiClient) -> Self {
        Self {
            state_manager,
            client,
            output: Arc::new(Mutex::new(output)),
            requests_per_second: NonZeroU32::new(8).unwrap(),
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let limiter = &RateLimiter::direct(Quota::per_second(
            self.requests_per_second,
        ));
        
        let after_id = self.state_manager.last_tag_id().await;
        let tags = futures::stream::unfold(after_id, |after_id| async move {
            
            // Wait until the rate limiter is ready
            limiter.until_ready().await;

            let response = self.client.query_tags_backoff(after_id).await;
            match response {
                Ok(response) => {
                    let tag_count = response.tags.len();
                    let highest_id = response
                        .tags
                        .iter()
                        .max_by_key(|tag| tag.id)
                        .map(|tag| tag.id)
                        .unwrap_or(0);
                    self.state_manager.update_last_tag_id(highest_id).await;
                    let output_lock = &mut *self.output.lock().await;
                    response.tags.into_iter().rev().for_each(|tag| {
                        self.process_tag(output_lock, tag.into());
                    });

                    info!("Downloaded after_id={}, Got {} Tags", after_id, tag_count);

                    Some(((), highest_id))
                }
                Err(e) => {
                    error!(
                        "Got error while scraping tags: {} at after_id={}",
                        e, after_id
                    );
                    self.state_manager
                        .append_error(ScrapeError::Tag(after_id))
                        .await;
                    None
                }
            }
        });

        // Consuming the stream to completion
        tags.count().await;

        Ok(())
    }

    pub fn process_tag(&self, output: &mut W, tag: Tag) {
        serde_json::to_writer(&mut *output, &tag).expect("Failed to write to output");
        output.write_all(b"\n").expect("Failed to write to output");
    }

}