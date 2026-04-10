use anyhow::{Context, Result};
use aws_sdk_s3::{primitives::ByteStream, Client};

use crate::uploads::s3_client::S3Settings;

#[derive(Clone)]
pub struct UploadService {
    client: Client,
    settings: S3Settings,
}

impl UploadService {
    pub fn new(client: Client, settings: S3Settings) -> Self {
        Self { client, settings }
    }

    pub async fn ensure_bucket_exists(&self) -> Result<()> {
        let bucket = self.settings.bucket.clone();

        let exists = self.client.head_bucket().bucket(&bucket).send().await.is_ok();

        if exists {
            return Ok(());
        }

        self.client
            .create_bucket()
            .bucket(&bucket)
            .send()
            .await
            .with_context(|| format!("failed to create S3 bucket '{bucket}'"))?;

        Ok(())
    }

    pub async fn upload_bytes(
        &self,
        key: &str,
        bytes: Vec<u8>,
        content_type: Option<&str>,
    ) -> Result<String> {
        let mut request = self
            .client
            .put_object()
            .bucket(&self.settings.bucket)
            .key(key)
            .body(ByteStream::from(bytes));

        if let Some(content_type) = content_type {
            request = request.content_type(content_type);
        }

        request
            .send()
            .await
            .with_context(|| format!("failed to upload object with key '{key}'"))?;

        Ok(self.public_url(key))
    }

    pub async fn delete_object(&self, key: &str) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.settings.bucket)
            .key(key)
            .send()
            .await
            .with_context(|| format!("failed to delete object with key '{key}'"))?;

        Ok(())
    }

    pub fn public_url(&self, key: &str) -> String {
        if let Some(base) = &self.settings.public_base_url {
            return format!("{}/{}", trim_suffix(base, '/'), trim_prefix(key, '/'));
        }

        format!(
            "{}/{}/{}",
            trim_suffix(&self.settings.endpoint, '/'),
            self.settings.bucket,
            trim_prefix(key, '/')
        )
    }
}

fn trim_prefix(value: &str, prefix: char) -> &str {
    value.strip_prefix(prefix).unwrap_or(value)
}

fn trim_suffix(value: &str, suffix: char) -> &str {
    value.strip_suffix(suffix).unwrap_or(value)
}
