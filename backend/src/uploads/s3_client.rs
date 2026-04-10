use std::env;

use anyhow::{Context, Result};
use aws_config::BehaviorVersion;
use aws_credential_types::{provider::SharedCredentialsProvider, Credentials};
use aws_sdk_s3::{config::Region, Client};

#[derive(Debug, Clone)]
pub struct S3Settings {
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub public_base_url: Option<String>,
}

impl S3Settings {
    pub fn from_env() -> Result<Self> {
        let endpoint = env::var("S3_ENDPOINT").context("S3_ENDPOINT is required")?;
        let region = env::var("S3_REGION").context("S3_REGION is required")?;
        let bucket = env::var("S3_BUCKET").context("S3_BUCKET is required")?;
        let access_key_id = env::var("AWS_ACCESS_KEY_ID").context("AWS_ACCESS_KEY_ID is required")?;
        let secret_access_key =
            env::var("AWS_SECRET_ACCESS_KEY").context("AWS_SECRET_ACCESS_KEY is required")?;
        let public_base_url = env::var("S3_PUBLIC_BASE_URL").ok();

        Ok(Self {
            endpoint,
            region,
            bucket,
            access_key_id,
            secret_access_key,
            public_base_url,
        })
    }
}

pub async fn build_client(settings: &S3Settings) -> Result<Client> {
    let credentials = Credentials::new(
        settings.access_key_id.clone(),
        settings.secret_access_key.clone(),
        None,
        None,
        "env",
    );

    let shared_config = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new(settings.region.clone()))
        .credentials_provider(SharedCredentialsProvider::new(credentials))
        .load()
        .await;

    let config = aws_sdk_s3::config::Builder::from(&shared_config)
        .endpoint_url(settings.endpoint.clone())
        .force_path_style(true)
        .build();

    Ok(Client::from_conf(config))
}
