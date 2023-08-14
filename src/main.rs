use anyhow::{anyhow, bail, Error};

use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
    process,
};

mod cache;
mod config;
mod download;
mod execute;
mod extract;

use cache::{get_binary_path, get_cache_dir};
use config::{load_config, ArchiveSource, PlatformConfig, HASHBANG_BINARY, HASHBANG_CONFIG_URL};
use download::download_url;
use execute::execute_command;
use extract::extract;

// The shell passes #! script name as first argument, so we pass other hashbang args via env vars
const HASHBANG_CACHE: &str = "HASHBANG_CACHE";
const HASHBANG_TARGET: &str = "HASHBANG_TARGET";

// What underlying binary are we trying to run from the launcher?
fn get_binary_name(
    binary_name_override: Option<String>,
    invoked_as: Option<OsString>,
    platform_config: &PlatformConfig,
) -> Result<String, Error> {
    if let Some(binary_name) = binary_name_override {
        Ok(binary_name)
    } else if platform_config.binaries.len() == 1 {
        return Ok(platform_config.binaries.keys().next().unwrap().to_string());
    } else if let Some(invoked_as) = invoked_as {
        let base_name = Path::new(&invoked_as).file_name();
        match base_name {
            Some(base_name) if &base_name.to_string_lossy() == "hashbang" => {
                bail!("{} env var not set to chose which binary", HASHBANG_BINARY)
            }
            Some(base_name) => Ok(base_name.to_str().unwrap().to_string()),
            None => bail!("Cannot determine binary base name from {:?}", invoked_as),
        }
    } else {
        bail!("No $0 found in args, this is unexpected")
    }
}

fn select_platform(
    platforms: &[config::PlatformConfig],
    target: String,
) -> Result<&config::PlatformConfig, Error> {
    for p in platforms {
        match target_spec::eval(&p.target_spec, &target)? {
            Some(true) => return Ok(p),
            Some(false) => continue,
            None => bail!("Invalid target spec: {}", p.target_spec),
        }
    }
    bail!("No matching platform found for {}", target)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config_url = if let Ok(config_url) = env::var(HASHBANG_CONFIG_URL) {
        config_url
    } else {
        bail!(
            "No config url specified, please set {}",
            HASHBANG_CONFIG_URL
        )
    };

    let target = if let Ok(target) = env::var(HASHBANG_TARGET) {
        target
    } else {
        env!("TARGET").to_string()
    };

    let binary_name_override = env::var(HASHBANG_BINARY).map(Some).unwrap_or(None);
    let cache_dir_override = env::var_os(HASHBANG_CACHE).map(PathBuf::from);

    let mut args = env::args_os();
    let invoked_as = args.next();
    let args = args.collect::<Vec<OsString>>();

    let (config, script_mode) = load_config(&args, &config_url).await?;
    let platform_config = select_platform(&config.platforms, target)?;

    let binary_name = get_binary_name(binary_name_override, invoked_as, platform_config)?;

    let binary_config = platform_config.binaries.get(&binary_name).unwrap();
    let archive_config = platform_config
        .archives
        .get(binary_config.provided_by.as_str())
        .ok_or_else(|| anyhow!("No archive config found for {}", binary_config.provided_by))?;

    let archive_cache_dir =
        get_cache_dir(cache_dir_override, &config.name, &archive_config.source)?;

    let binary_path = get_binary_path(&archive_cache_dir, &binary_name, binary_config);

    let need_download = !(binary_path.exists() && binary_path.is_file());

    if need_download {
        let archive_bytes = match &archive_config.source {
            ArchiveSource::Url(url) => {
                eprintln!("Downloading {:?}", &url);
                download_url(url.clone()).await?
            }
        };
        eprintln!("Extracting to {:?}", &archive_cache_dir);
        extract(
            &archive_cache_dir,
            archive_bytes,
            archive_config.extract_steps.clone(),
        )?;

        if !binary_path.exists() {
            bail!(
                "{} does not exist even after download",
                binary_path.display()
            )
        }
    }

    let status = execute_command(&binary_path, env::vars_os(), args.into_iter(), script_mode);

    if !status.success() {
        process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
