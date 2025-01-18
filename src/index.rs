use std::{collections::HashMap, path::Path};

use rayon::{iter::ParallelIterator, str::ParallelString};
use roaring::RoaringBitmap;
use serde::{Deserialize, Serialize};

use crate::models::{Post, PostSimplified, Tag};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Index {
    pub tag_str_to_id: HashMap<String, u32>,
    pub tag_id_to_post_id: HashMap<u32, RoaringBitmap>,
    pub post_id_to_post: HashMap<u32, PostSimplified>,
    pub tag_id_freq: HashMap<u32, u32>,
}

impl Index {
    pub fn generate(post_file: &str, tag_file: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut index = Index::default();
        let tags = std::fs::read_to_string(tag_file)?;
        let tags: Vec<Tag> = tags
            .par_lines()
            .map(serde_json::from_str)
            .flatten()
            .collect();

        for tag in tags {
            index.insert_tag(tag);
        }

        let posts = std::fs::read_to_string(post_file)?;
        posts
            .lines()
            .flat_map(serde_json::from_str)
            .for_each(|post| index.insert_post(post));

        Ok(index)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = std::fs::File::create(path)?;
        let writer = std::io::BufWriter::new(file);
        serde_json::to_writer(writer, self)?;
        Ok(())
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let index = serde_json::from_reader(reader)?;
        Ok(index)
    }

    pub fn insert_tag(&mut self, tag: Tag) {
        self.tag_str_to_id
            .insert(tag.name.to_lowercase(), tag.id as u32);
    }

    pub fn insert_post(&mut self, post: Post) {
        for tag in post.split_tags() {
            let tag = tag.to_lowercase();
            let tag_id = match self.tag_str_to_id.get(&tag) {
                Some(id) => id,
                None => continue,
            };
            let bitmap = self.tag_id_to_post_id.entry(*tag_id).or_default();
            if bitmap.insert(post.id as u32) {
                *self.tag_id_freq.entry(*tag_id).or_default() += 1;
            }
        }
        self.post_id_to_post.insert(post.id as u32, post.into());
    }

    pub fn get_post_ids_by_tag(&self, tag: &str) -> Option<RoaringBitmap> {
        let tag_id = self.tag_str_to_id.get(tag)?;
        let image_ids = self.tag_id_to_post_id.get(tag_id)?.clone();
        Some(image_ids)
    }

    pub fn get_images_all_tags_lazy(
        &self,
        tags: impl IntoIterator<Item = String>,
    ) -> Option<impl Iterator<Item = PostSimplified> + '_> {
        let mut tag_data: Vec<(u32, u32)> = tags
            .into_iter()
            .filter_map(|tag| {
                let tag_id = self.tag_str_to_id.get(&tag)?;
                let frequency = self.tag_id_freq.get(tag_id).copied().unwrap_or(u32::MAX);
                Some((*tag_id, frequency))
            })
            .collect();

        tag_data.sort_by_key(|(_, freq)| *freq);

        let mut tag_data = tag_data.into_iter();

        let mut result = self.tag_id_to_post_id.get(&tag_data.next()?.0)?.clone();

        for (tag_id, _) in tag_data {
            let next_set = self.tag_id_to_post_id.get(&tag_id)?;
            result &= next_set;
            if result.is_empty() {
                return None; // Early exit if intersection becomes empty
            }
        }

        Some(
            result
                .into_iter() // Iterate over the resulting post IDs
                .filter_map(move |id| self.post_id_to_post.get(&id).cloned()), // Lazily map IDs to PostSimplified
        )
    }
}
