use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::s3::S3ClientRef;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostMetadata {
    pub title: String,
    pub slug: String,
    pub summary: Option<String>,
    pub author: Option<String>,
    pub date: DateTime<Utc>,
    pub date_rfc3339: String,
    pub date_display: String,
    pub tags: Vec<String>,
    pub cover_image: Option<String>,
}

impl PostMetadata {
    pub fn from_frontmatter(slug: String, frontmatter: &serde_json::Value) -> AppResult<Self> {
        let title = frontmatter
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or(AppError::InvalidFrontmatter)?
            .to_string();

        let summary = frontmatter
            .get("summary")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let author = frontmatter
            .get("author")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let date_str = frontmatter
            .get("date")
            .and_then(|v| v.as_str())
            .ok_or(AppError::InvalidFrontmatter)?;

        let date = DateTime::parse_from_rfc3339(date_str)
            .or_else(|_| DateTime::parse_from_str(date_str, "%Y-%m-%d"))
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|_| AppError::InvalidFrontmatter)?;

        let date_rfc3339 = date.to_rfc3339();
        let date_display = date.format("%B %d, %Y").to_string();

        let tags = frontmatter
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let cover_image = frontmatter
            .get("cover_image")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(Self {
            title,
            slug,
            summary,
            author,
            date,
            date_rfc3339,
            date_display,
            tags,
            cover_image,
        })
    }
}

#[derive(Clone)]
pub struct Cache {
    posts: Arc<RwLock<Vec<PostMetadata>>>,
    slug_index: Arc<RwLock<HashMap<String, usize>>>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            posts: Arc::new(RwLock::new(Vec::new())),
            slug_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn refresh(&self, s3_client: &S3ClientRef, _config: &Config) -> AppResult<()> {
        let keys = s3_client.list_objects("markdown/").await?;

        let mut posts = Vec::new();
        let mut slug_index = HashMap::new();

        for key in keys {
            if !key.ends_with(".md") {
                continue;
            }

            let slug = key.strip_prefix("markdown/").unwrap_or(&key).strip_suffix(".md").unwrap_or(&key);

            if slug.is_empty() {
                continue;
            }

            let content = s3_client.get_object(&key).await?;

            let (frontmatter, _) = parse_frontmatter(&content)?;

            if let Ok(post) = PostMetadata::from_frontmatter(slug.to_string(), &frontmatter) {
                let index = posts.len();
                slug_index.insert(post.slug.clone(), index);
                posts.push(post);
            }
        }

        posts.sort_by(|a, b| b.date.cmp(&a.date));

        {
            let mut posts_lock = self.posts.write().unwrap();
            *posts_lock = posts;
        }

        {
            let mut index_lock = self.slug_index.write().unwrap();
            *index_lock = slug_index;
        }

        Ok(())
    }

    pub async fn get_posts(&self) -> Vec<PostMetadata> {
        self.posts.read().unwrap().clone()
    }

    pub async fn get_post_by_slug(&self, slug: &str) -> Option<PostMetadata> {
        let index_lock = self.slug_index.read().unwrap();
        let posts_lock = self.posts.read().unwrap();

        index_lock.get(slug).and_then(|&i| posts_lock.get(i).cloned())
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_frontmatter(content: &str) -> AppResult<(serde_json::Value, String)> {
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return Err(AppError::InvalidFrontmatter);
    }
    let frontmatter_str = parts[1].trim();
    let body = parts[2].trim().to_string();
    let frontmatter: serde_json::Value = serde_yaml::from_str(frontmatter_str)?;
    Ok((frontmatter, body))
}