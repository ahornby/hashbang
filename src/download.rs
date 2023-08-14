use anyhow::Error;
use bytes::Bytes;
use url::Url;

// simple url fetch case
pub(crate) async fn download_url(url: Url) -> Result<Bytes, Error> {
    reqwest::get(url).await?.bytes().await.map_err(Into::into)
}
