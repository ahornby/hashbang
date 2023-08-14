use anyhow::{anyhow, Context, Error};
use serde::{Deserialize, Serialize};
use url::Url;

use std::{collections::HashMap, ffi::OsString, fs, path::PathBuf};

// The shell passes #! script name as first argument, so we pass other hashbang args via env vars
pub const HASHBANG_BINARY: &str = "HASHBANG_BINARY";

/// env var to point to toml config defined in `Config`.  can use data: for inline,  file:, or https: schemes
pub const HASHBANG_CONFIG_URL: &str = "HASHBANG_CONFIG_URL";

const DATA_SCHEME: &str = "data:";
const FILE_SCHEME: &str = "file:";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Where to download from
pub enum ArchiveSource {
    Url(Url),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Some tools ship as a single file, others as a compressed directory
pub enum ExtractStep {
    /// Make a binary executable
    MakeExecutable(String),
    /// Decompresses a single archive file
    ZstdDecompress,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Archives contain binaries, they are downloaded from sources, then extracted by steps into the cache area
pub struct ArchiveConfig {
    pub source: ArchiveSource,
    pub extract_steps: Vec<ExtractStep>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Binaries are runnable from the expanded archive in the cache area
pub struct BinaryConfig {
    pub provided_by: String,
    pub sub_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Chose what to do on a given platform
pub struct PlatformConfig {
    /// Target spec format listed at <https://crates.io/crates/target-spec>
    pub target_spec: String,
    pub archives: HashMap<String, ArchiveConfig>,
    pub binaries: HashMap<String, BinaryConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Top level config
pub struct Config {
    /// Simple name for the config e.g. "buck2" or "rustc-stable"
    pub name: String,
    /// Toml list rather than map so you can put more specific match first if needed
    /// e.g. x86_64-apple-darwin and x86_64-unknown-linux-gnu before cfg(all(unix, target_arch = "x86_64"))
    pub platforms: Vec<PlatformConfig>,
}

// When running from #! this tells caller to consumes the first argument as the path to the config by returning (_, true)
pub(crate) async fn load_config(
    args: &[OsString],
    config_url: &str,
) -> Result<(Config, bool), Error> {
    if config_url.starts_with(DATA_SCHEME) {
        // Short circuit if the user has given us a config in the environment
        return Ok((
            toml::from_str(config_url.trim_start_matches(DATA_SCHEME))?,
            false,
        ));
    }
    let mut script_mode = false;

    let config = if config_url.starts_with(FILE_SCHEME) {
        let path = config_url.trim_start_matches(FILE_SCHEME);
        let path = if path == "." {
            // We are reading current script via a #! line, which passes the filename in first arg
            script_mode = true;
            args.iter().next().map(PathBuf::from).ok_or_else(|| {
                anyhow!("First arg should be path to script, but no args found, this is unexpected")
            })?
        } else {
            PathBuf::from(path)
        };
        fs::read_to_string(&path).with_context(|| anyhow!("can't read {:?}", path))?
    } else {
        let url = Url::parse(config_url)?;
        let bytes = reqwest::get(url).await?.bytes().await?;
        String::from_utf8(bytes.to_vec())?
    };

    toml::from_str(&config)
        .map_err(Into::into)
        .map(|v| (v, script_mode))
}

#[cfg(test)]
mod test {
    use super::*;
    use std::env;

    const SAMPLE_CONFIG: &str = r#"
    name = "buck2-test-config"
    
    [[platforms]]
    target_spec = "x86_64-apple-darwin"
    archives."buck2_zst".source.url = "https://github.com/facebook/buck2/releases/download/2023-08-01/buck2-x86_64-apple-darwin.zst"
    archives."buck2_zst".extract_steps = [ "zstd_decompress", { make_executable = "buck2" }, ]
    binaries."buck2".provided_by = "buck2_zst"

    [[platforms]]
    target_spec = "x86_64-unknown-linux-gnu"
    archives."buck2_zst".source.url = "https://github.com/facebook/buck2/releases/download/2023-08-01/buck2-x86_64-unknown-linux-gnu.zst"
    archives."buck2_zst".extract_steps = [ "zstd_decompress", { make_executable = "buck2" }, ]
    binaries."buck2".provided_by = "buck2_zst"

    "#;

    #[test]
    fn test_config_parse() {
        let _parsed: Config = toml::from_str(SAMPLE_CONFIG).unwrap();
    }

    #[tokio::test]
    async fn test_config_url_parse() {
        let (_parsed, _script_mode): (Config, bool) = load_config(
            &env::args_os().collect::<Vec<OsString>>(),
            &format!("{}{}", DATA_SCHEME, SAMPLE_CONFIG),
        )
        .await
        .unwrap();
    }
}
