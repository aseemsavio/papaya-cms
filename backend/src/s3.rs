use std::sync::Arc;

use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_s3::config::{Credentials as SdkCredentials, Region};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client as S3SdkClient;

use crate::config::Config;
use crate::error::{AppError, AppResult};

pub struct S3Client {
    client: S3SdkClient,
    bucket: String,
    root_folder: String,
}

impl S3Client {
    pub async fn new(config: &Config) -> AppResult<Self> {
        let credentials = Credentials::new(
            &config.s3.access_key,
            &config.s3.secret_key,
            None,
            None,
            "papaya-cms",
        );

        let sdk_credentials = SdkCredentials::new(
            credentials.access_key_id(),
            credentials.secret_access_key(),
            credentials.session_token().map(|s| s.to_string()),
            None,
            "papaya-cms",
        );

        let region = Region::new(config.s3.region.clone());

        let mut s3_config = aws_sdk_s3::config::Builder::new()
            .region(region)
            .credentials_provider(sdk_credentials)
            .behavior_version(BehaviorVersion::latest())
            .endpoint_url(&config.s3.endpoint);

        if config.s3.force_path_style {
            s3_config = s3_config.force_path_style(true);
        }

        let client = S3SdkClient::from_conf(s3_config.build());

        Ok(Self {
            client,
            bucket: config.s3.bucket.clone(),
            root_folder: config.s3.root_folder.trim_matches('/').to_string(),
        })
    }

    /// Maps a site-relative key (e.g. `markdown/post.md`) to its bucket key
    /// (e.g. `blog-a/markdown/post.md`).
    fn full_key(&self, key: &str) -> String {
        format!("{}/{}", self.root_folder, key)
    }

    pub async fn list_objects(&self, prefix: &str) -> AppResult<Vec<String>> {
        let mut keys = Vec::new();
        let mut continuation_token: Option<String> = None;
        let full_prefix = self.full_key(prefix);
        let root_prefix = format!("{}/", self.root_folder);

        loop {
            let mut request = self
                .client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(&full_prefix);

            if let Some(token) = continuation_token {
                request = request.continuation_token(token);
            }

            let response = request.send().await.map_err(|e| AppError::S3(e.to_string()))?;

            if let Some(contents) = response.contents {
                for object in contents {
                    if let Some(key) = object.key {
                        let relative = key.strip_prefix(&root_prefix).unwrap_or(&key);
                        keys.push(relative.to_string());
                    }
                }
            }

            if response.is_truncated.unwrap_or(false) {
                continuation_token = response.next_continuation_token;
            } else {
                break;
            }
        }

        Ok(keys)
    }

    pub async fn get_object(&self, key: &str) -> AppResult<String> {
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(self.full_key(key))
            .send()
            .await
            .map_err(|e| AppError::S3(e.to_string()))?;

        let body = response.body.collect().await.map_err(|e| AppError::S3(e.to_string()))?;
        let bytes = body.into_bytes();
        let content = String::from_utf8(bytes.to_vec()).map_err(|e| AppError::S3(e.to_string()))?;

        Ok(content)
    }

    pub async fn get_object_bytes(&self, key: &str) -> AppResult<Vec<u8>> {
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(self.full_key(key))
            .send()
            .await
            .map_err(|e| AppError::S3(e.to_string()))?;

        let body = response.body.collect().await.map_err(|e| AppError::S3(e.to_string()))?;
        Ok(body.into_bytes().to_vec())
    }

    pub async fn put_object(&self, key: &str, data: Vec<u8>, content_type: &str) -> AppResult<()> {
        let body = ByteStream::from(data);
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(self.full_key(key))
            .body(body)
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| AppError::S3(e.to_string()))?;

        Ok(())
    }

    pub async fn delete_object(&self, key: &str) -> AppResult<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(self.full_key(key))
            .send()
            .await
            .map_err(|e| AppError::S3(e.to_string()))?;

        Ok(())
    }
}

pub type S3ClientRef = Arc<S3Client>;