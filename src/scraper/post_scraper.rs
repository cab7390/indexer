use super::state_manager::StateManager;
use crate::{
    api::{
        client::ApiClient,
        models::{ApiError, ApiPostResponse},
    },
    models::Post,
    scraper::state_manager::ScrapeError,
};
use futures::StreamExt;
use governor::{state::StreamRateLimitExt, Quota, RateLimiter};
use std::{io::Write, num::NonZeroU32, sync::Arc};
use tokio::sync::Mutex;
use tracing::{error, info};

pub struct PostScraper<W: Write> {
    state_manager: StateManager,
    client: ApiClient,
    output: Arc<Mutex<W>>,
    parallel_requests: usize,
    requests_per_second: u32,
}

impl<W: Write> PostScraper<W> {
    pub fn new(output: W, state_manager: StateManager, client: ApiClient) -> Self {
        Self {
            state_manager,
            client,
            output: Arc::new(Mutex::new(output)),
            parallel_requests: 2,
            requests_per_second: 8,
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let starting_id = self.state_manager.last_post_id().await + 1;
        let ranges = (starting_id..).step_by(100).map(|start| start..start + 100);
        let limiter = RateLimiter::direct(Quota::per_second(
            NonZeroU32::new(self.requests_per_second).unwrap(),
        ));
        let posts = futures::stream::iter(ranges)
            .map(|id_range| async {
                (
                    id_range.clone(),
                    self.client.query_posts_backoff(id_range).await,
                )
            })
            .buffered(self.parallel_requests)
            .ratelimit_stream(&limiter);

        posts
            .for_each(|(id_range, post)| async {
                self.process_response(id_range, post).await;
            })
            .await;

        Ok(())
    }

    pub async fn process_response(
        &self,
        id_range: std::ops::Range<u64>,
        result: Result<ApiPostResponse, ApiError>,
    ) {
        match result {
            Ok(result) => {
                if result.attributes.count == 0 {
                    return;
                }

                let highest_id = result
                    .posts
                    .iter()
                    .max_by_key(|post| post.id)
                    .map(|post| post.id)
                    .unwrap_or(0);

                self.state_manager.update_last_post_id(highest_id).await;
                let output_lock = &mut *self.output.lock().await;
                result.posts.into_iter().rev().for_each(|post| {
                    self.process_post(output_lock, post.into());
                });
                info!(
                    "Downloaded {:?}. Got: {} Posts",
                    id_range, result.attributes.count
                );
            }
            Err(e) => {
                self.state_manager
                    .append_error(ScrapeError::Post(id_range.clone()))
                    .await;
                error!(
                    "Got error while scraping posts: {} in id range: {:?}",
                    e, id_range
                );
            }
        }
    }

    pub fn process_post(&self, output: &mut W, post: Post) {
        serde_json::to_writer(&mut *output, &post).expect("Failed to write to output");
        output.write_all(b"\n").expect("Failed to write to output");
    }
}
