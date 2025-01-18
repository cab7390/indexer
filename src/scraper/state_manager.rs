use std::{ops::Range, path::Path, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ScrapeError {
    Post(Range<u64>),
    Tag(u64),
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScrapeState {
    pub last_post_id: u64,
    pub last_tag_id: u64,
    pub errors: Vec<ScrapeError>
}

/// Manages the state of the scraper across multiple threads
#[derive(Debug, Clone)]
pub struct StateManager {
    state: Arc<Mutex<ScrapeState>>
}

impl StateManager {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, serde_json::Error> {
        let state = match std::fs::File::open(path) {
            Ok(state_file) => serde_json::from_reader(state_file)?,
            Err(e) => {
                error!("Unable to open state file: {:?}", e);
                ScrapeState {
                    last_post_id: 0,
                    last_tag_id: 0,
                    errors: Vec::new(),
                }
            }
        };

        let state = Arc::new(Mutex::new(state));
        Ok(Self { state })
    }

    pub async fn update_last_post_id(&self, last_post_id: u64) {
        self.state.lock().await.last_post_id = last_post_id;
    }

    pub async fn update_last_tag_id(&self, last_tag_id: u64) {
        self.state.lock().await.last_tag_id = last_tag_id;
    }

    pub async fn last_post_id(&self) -> u64 {
        self.state.lock().await.last_post_id
    }

    pub async fn last_tag_id(&self) -> u64 {
        self.state.lock().await.last_tag_id
    }

    pub async fn append_error(&self, error: ScrapeError) {
        self.state.lock().await.errors.push(error);
    }

    pub fn get_state(&self) -> Arc<Mutex<ScrapeState>> {
        self.state.clone()
    }

    pub async fn save_state(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.state.lock().await;
        let file = std::fs::File::create(file_path)?;
        serde_json::to_writer(file, &*state)?;
        Ok(())
    }
}