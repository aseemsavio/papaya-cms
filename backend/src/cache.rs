use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

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
    pub date_year: String,
    pub tags: Vec<String>,
    pub cover_image: Option<String>,
    /// Free-form photography metadata (aperture, shutter, focal length, etc.)
    /// straight from the Markdown frontmatter. Always an object so themes can
    /// safely probe `post.photo.aperture` without guarding for null.
    pub photo: serde_json::Value,
}

impl PostMetadata {
    pub fn from_frontmatter(slug: String, frontmatter: &serde_json::Value) -> AppResult<Self> {
        let title = frontmatter
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                warn!("Missing or invalid 'title' field for slug: {}", slug);
                AppError::InvalidFrontmatter
            })?
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
            .ok_or_else(|| {
                warn!("Missing or invalid 'date' field for slug: {}", slug);
                AppError::InvalidFrontmatter
            })?;

        let date = DateTime::parse_from_rfc3339(date_str)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                    .map(|nd| nd.and_hms_opt(0, 0, 0).unwrap().and_utc())
            })
            .map_err(|e| {
                warn!("Date parse failed for slug '{}': {} (value: '{}')", slug, e, date_str);
                AppError::InvalidFrontmatter
            })?;

        let date_rfc3339 = date.to_rfc3339();
        let date_display = date.format("%B %d, %Y").to_string();
        let date_year = date.format("%Y").to_string();

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

        let photo = frontmatter
            .get("photo")
            .filter(|v| v.is_object())
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));

        Ok(Self {
            title,
            slug,
            summary,
            author,
            date,
            date_rfc3339,
            date_display,
            date_year,
            tags,
            cover_image,
            photo,
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
        info!("Refreshing cache from S3...");
        let keys = s3_client.list_objects("markdown/").await?;
        info!("Found {} objects in S3 under markdown/ prefix", keys.len());

        let mut posts = Vec::new();
        let mut loaded_count = 0;

        for key in keys {
            if !key.ends_with(".md") {
                continue;
            }

            let slug = key.strip_prefix("markdown/").unwrap_or(&key).strip_suffix(".md").unwrap_or(&key);

            if slug.is_empty() {
                continue;
            }

            let content = match s3_client.get_object(&key).await {
                Ok(c) => c,
                Err(e) => {
                    warn!("Failed to fetch object {}: {}", key, e);
                    continue;
                }
            };

            let (frontmatter, _) = match parse_frontmatter(&content) {
                Ok(fm) => fm,
                Err(e) => {
                    warn!("Failed to parse frontmatter for {}: {}", slug, e);
                    continue;
                }
            };

            match PostMetadata::from_frontmatter(slug.to_string(), &frontmatter) {
                Ok(post) => {
                    posts.push(post.clone());
                    loaded_count += 1;
                    info!("Loaded blog post: {} (title: {}, date: {})", post.slug, post.title, post.date_display);
                }
                Err(e) => {
                    warn!("Failed to build post metadata for {}: {}", slug, e);
                }
            }
        }

        // Sort newest-first BEFORE building the slug index, otherwise the stored
        // indices would point at the wrong posts after the reorder.
        posts.sort_by(|a, b| b.date.cmp(&a.date));

        let slug_index: HashMap<String, usize> = posts
            .iter()
            .enumerate()
            .map(|(index, post)| (post.slug.clone(), index))
            .collect();

        {
            let mut posts_lock = self.posts.write().unwrap();
            *posts_lock = posts;
        }

        {
            let mut index_lock = self.slug_index.write().unwrap();
            *index_lock = slug_index;
        }

        info!("Cache refresh completed. Loaded {} blog posts.", loaded_count);
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