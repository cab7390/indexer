use std::ops::Range;

use typed_builder::TypedBuilder;

use super::models::{ApiError, ApiPostResponse, ApiTagResponse};


#[derive(Debug, Clone, TypedBuilder)]
pub struct ApiClient {
    #[builder(default)]
    pub client: reqwest::Client,

    #[builder(setter(into, strip_option))]
    pub api_key: Option<String>,
    
    #[builder(setter(into, strip_option))]
    pub user_id: Option<String>,

    #[builder(setter(into))]
    pub endpoint: String,
}

impl ApiClient {

    /// Add the api_key and user_id to the request
    fn add_credentials(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let mut params = Vec::new();

        if let Some(api_key) = &self.api_key {
            params.push(("api_key", api_key));
        }

        if let Some(user_id) = &self.user_id {
            params.push(("user_id", user_id));
        }

        req.query(&params)
    }

    /// Query the posts
    async fn query_posts(&self, id: Range<u64>) -> Result<ApiPostResponse, ApiError> {
        let req = self.client.get(&self.endpoint).query(&[
            ("page", "dapi"),
            ("s", "post"),
            ("q", "index"),
            ("json", "1"),
            ("limit", "100"),
        ]);

        // Add the api_key and user_id to the request
        let req = self.add_credentials(req);

        // Create the id range part of the request
        let tags = format!("id:>={} id:<{}", id.start, id.end);
        let request = req.query(&[("tags", tags)]);

        Ok(request.send().await?.json().await?)
    }

    /// Query the posts with a backoff strategy
    pub async fn query_posts_backoff(&self, id: Range<u64>) -> Result<ApiPostResponse, ApiError> {
        backoff::future::retry(backoff::ExponentialBackoff::default(), || async {
            Ok(self.query_posts(id.clone()).await?)
        }).await
    }
    
    /// Query the tags
    async fn query_tags(&self, after_id: u64) -> Result<ApiTagResponse, ApiError> {
        let req = self.client.get("https://gelbooru.com/index.php").query(&[
            ("page", "dapi"),
            ("s", "tag"),
            ("q", "index"),
            ("json", "1"),
            ("limit", "100"),
            ("order", "asc"),
            ("orderby", "id"),
            ("after_id", &format!("{after_id}")),
        ]);

        let req = self.add_credentials(req);

        Ok(req.send().await?.json().await?)
    }

    /// Query the tags with a backoff strategy
    pub async fn query_tags_backoff(&self, after_id: u64) -> Result<ApiTagResponse, ApiError> {
        backoff::future::retry(backoff::ExponentialBackoff::default(), || async {
            Ok(self.query_tags(after_id).await?)
        }).await
    }

}