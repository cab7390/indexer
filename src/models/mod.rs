use std::fmt::Debug;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::api::models::{ApiPost, ApiTag};

#[derive(Debug, Clone, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub enum Rating {
    Safe,         // safe, general
    Sensitive,    // sensitive
    Questionable, // questionable
    Explicit,     // explicit
                  // Other(String), // just in case
}

impl From<String> for Rating {
    fn from(value: String) -> Self {
        match value.as_str() {
            "safe" | "general" => Rating::Safe,
            "sensitive" => Rating::Sensitive,
            "questionable" => Rating::Questionable,
            "explicit" => Rating::Explicit,
            _ => panic!("invalid value for rating"),
        }
    }
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub struct Varient {
    pub url: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub struct Post {
    pub id: u64,
    pub created_at: DateTime<Utc>,
    pub score: i32,
    pub md5: String,
    pub directory: String,
    pub image: String,
    pub rating: Rating,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub change: u64,
    pub owner: String,
    pub creator_id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample: Option<Varient>,
    pub preview: Varient,
    pub original: Varient,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub has_notes: bool,
    pub has_comments: bool,
    pub status: String,
    pub post_locked: bool,
    pub has_children: bool,
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub enum Extension {
    Png,
    Jpg,
    Jpeg,
    Gif,
    Mov,
    Other(String),
}

impl From<String> for Extension {
    fn from(value: String) -> Self {
        match value.as_str() {
            "png" => Self::Png,
            "jpg" => Self::Jpg,
            "jpeg" => Self::Jpeg,
            "gif" => Self::Gif,
            "mov" => Self::Mov,
            _ => Self::Other(value),
        }
    }
}

impl Extension {
    pub fn as_str(&self) -> &str {
        match self {
            Extension::Png => "png",
            Extension::Jpg => "jpg",
            Extension::Jpeg => "jpeg",
            Extension::Gif => "gif",
            Extension::Mov => "mov",
            Extension::Other(v) => v.as_str(),
        }
    }
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct PostSimplified {
    pub md5: [u8; 16],
    pub extension: Extension,
    pub id: u32,
    pub created_at: DateTime<Utc>,
}

impl From<Post> for PostSimplified {
    fn from(value: Post) -> Self {
        let mut hash = [0u8; 16];
        let extension = Extension::from(value.image.rsplit_once('.').unwrap().1.to_string());
        hex::decode_to_slice(value.md5, &mut hash).unwrap();
        Self {
            md5: hash,
            extension,
            id: value.id as u32,
            created_at: value.created_at,
        }
    }
}

impl Post {
    pub fn split_tags(&self) -> impl Iterator<Item = &str> {
        self.tags.iter().map(|tag| tag.as_str()) // wrong change tags back to string
    }
}

impl From<ApiPost> for Post {
    fn from(value: ApiPost) -> Self {
        let sample = match (value.sample_url, value.sample_width, value.sample_height) {
            (Some(url), Some(width), Some(height)) => Some(Varient { url, width, height }),
            _ => None,
        };

        Post {
            id: value.id,
            created_at: value.created_at,
            score: value.score,
            md5: value.md5,
            directory: value.directory,
            image: value.image,
            rating: Rating::from(value.rating),
            source: value.source,
            change: value.change,
            owner: value.owner,
            creator_id: value.creator_id,
            parent_id: value.parent_id,
            sample,
            preview: Varient {
                url: value.preview_url,
                width: value.preview_width,
                height: value.preview_height,
            },
            original: Varient {
                url: value.file_url,
                width: value.width,
                height: value.height,
            },
            tags: value
                .tags
                .split_whitespace()
                .map(|tag| tag.to_string())
                .collect(),
            title: value.title,
            has_notes: value.has_notes,
            has_comments: value.has_comments,
            status: value.status,
            post_locked: value.post_locked,
            has_children: value.has_children,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize)]
pub enum TagType {
    Artist,
    Character,
    Copyright,
    Metadata,
    Descriptive,
    Other(u32),
}

impl From<u32> for TagType {
    fn from(value: u32) -> Self {
        match value {
            0 => TagType::Descriptive,
            1 => TagType::Artist,
            3 => TagType::Copyright,
            4 => TagType::Character,
            5 => TagType::Metadata,
            v => TagType::Other(v),
        }
    }
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct Tag {
    pub id: u64,
    pub name: String,
    pub count: u64,
    pub tag_type: TagType,
    pub ambiguous: bool,
}

impl From<ApiTag> for Tag {
    fn from(value: ApiTag) -> Self {
        Tag {
            id: value.id,
            name: value.name,
            count: value.count,
            tag_type: TagType::from(value.tag_type),
            ambiguous: value.ambiguous,
        }
    }
}
