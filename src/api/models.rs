use chrono::{DateTime, Utc};
use serde::Deserialize;
use thiserror::Error;

use crate::api::utils::{api_bool, api_date, api_option_str, api_option_u32, api_option_u64};

#[derive(Debug, Clone, Deserialize)]
pub struct ApiPostResponse {
    #[serde(rename = "@attributes")]
    pub attributes: ApiAttributes,
    #[serde(default, rename = "post")]
    pub posts: Vec<ApiPost>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiAttributes {
    pub limit: u64,
    pub offset: u64,
    pub count: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiPost {
    pub id: u64,
    #[serde(deserialize_with = "api_date")]
    pub created_at: DateTime<Utc>,
    pub score: i32,
    pub width: u32,
    pub height: u32,
    pub md5: String,
    pub directory: String,
    pub image: String,
    pub rating: String,
    #[serde(deserialize_with = "api_option_str")]
    pub source: Option<String>,
    pub change: u64,
    pub owner: String,
    pub creator_id: u64,
    #[serde(deserialize_with = "api_option_u64")]
    pub parent_id: Option<u64>,
    #[serde(deserialize_with = "api_bool")]
    pub sample: bool,
    pub preview_height: u32,
    pub preview_width: u32,
    pub tags: String,
    #[serde(deserialize_with = "api_option_str")]
    pub title: Option<String>,
    #[serde(deserialize_with = "api_bool")]
    pub has_notes: bool,
    #[serde(deserialize_with = "api_bool")]
    pub has_comments: bool,
    pub file_url: String,
    pub preview_url: String,
    #[serde(deserialize_with = "api_option_str")]
    pub sample_url: Option<String>,
    #[serde(deserialize_with = "api_option_u32")]
    pub sample_height: Option<u32>,
    #[serde(deserialize_with = "api_option_u32")]
    pub sample_width: Option<u32>,
    pub status: String,
    #[serde(deserialize_with = "api_bool")]
    pub post_locked: bool,
    #[serde(deserialize_with = "api_bool")]
    pub has_children: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiTagResponse {
    #[serde(rename = "@attributes")]
    pub attributes: ApiAttributes,
    #[serde(rename = "tag")]
    pub tags: Vec<ApiTag>,
}


#[derive(Debug, Clone, Deserialize)]
pub struct ApiTag {
    pub id: u64,
    pub name: String,
    pub count: u64,
    #[serde(rename="type")]
    pub tag_type: u32,
    #[serde(deserialize_with = "api_bool")]
    pub ambiguous: bool,
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Reqwest Error: `{0}`")]
    Reqwest(#[from] reqwest::Error),
    #[error("Serde Error: `{0}`")]
    Serde(#[from] serde_json::Error),
    #[error("Other")]
    Other
}