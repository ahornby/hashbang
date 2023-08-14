use anyhow::{anyhow, Error};
use directories::BaseDirs;

use std::path::{Path, PathBuf};

use crate::config::{ArchiveSource, BinaryConfig};

pub(crate) fn get_cache_dir(
    cache_override: Option<PathBuf>,
    config_name: &str,
    archive_source: &ArchiveSource,
) -> Result<PathBuf, Error> {
    let cache_dir = if let Some(cache_override) = cache_override {
        cache_override
    } else {
        let base_dirs = BaseDirs::new().ok_or_else(|| anyhow!("No home directory"))?;
        base_dirs.cache_dir().to_path_buf()
    };
    let cache_dir = cache_dir.join("hashbang").join(config_name);
    let archive_cache_digest = match &archive_source {
        ArchiveSource::Url(url) => sha256::digest(url.as_str()),
    };
    let cache_dir = cache_dir.join(archive_cache_digest);
    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir)?;
    }
    Ok(cache_dir)
}

pub(crate) fn get_binary_path(
    cache_dir: &Path,
    binary_name: &str,
    binary_config: &BinaryConfig,
) -> PathBuf {
    if let Some(sub_path) = &binary_config.sub_path {
        // makes sense once we have multi file archines
        cache_dir.join(sub_path)
    } else {
        cache_dir.join(binary_name)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use url::Url;

    #[test]
    fn test_get_cache_dir() {
        let d = get_cache_dir(
            None,
            "test",
            &ArchiveSource::Url(Url::parse("https://localhost/").unwrap()),
        )
        .unwrap();
        #[cfg(target_os = "macos")]
        assert!(d.to_str().unwrap().ends_with("/Library/Caches/hashbang/test/f2b99ce05b94599549c70dbbe7a891b278e7c3cacad02334fa44682fca36c740"));
    }
}
