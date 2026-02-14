use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use sui_config::sui_config_dir;
use sui_sdk::wallet_context::WalletContext;

/// Load wallet context from a user-provided config path
pub fn load_wallet_context(config_path: impl AsRef<Path>) -> Result<WalletContext> {
    let config_path = config_path.as_ref();

    if !config_path.exists() {
        return Err(anyhow!(
            "Config file not found at: {}. Please ensure the path points to a valid Sui client config file.",
            config_path.display()
        ));
    }

    if !config_path.is_file() {
        return Err(anyhow!(
            "Path must point to a file, not a directory: {}",
            config_path.display()
        ));
    }

    let wallet =
        WalletContext::new(config_path)?.with_request_timeout(std::time::Duration::from_secs(60));

    Ok(wallet)
}

/// Accept directory path and construct config file path
pub fn load_wallet_from_directory(dir_path: impl AsRef<Path>) -> Result<WalletContext> {
    let dir = dir_path.as_ref();

    if !dir.exists() || !dir.is_dir() {
        return Err(anyhow!("Invalid directory path: {}", dir.display()));
    }

    let config_file = dir.join("client.yaml");

    if !config_file.exists() {
        return Err(anyhow!(
            "No client.yaml found in directory: {}. Expected config at: {}",
            dir.display(),
            config_file.display()
        ));
    }

    load_wallet_context(config_file)
}

pub fn default_wallet_config() -> Result<PathBuf> {
    Ok(sui_config_dir()?.join("client.yaml"))
}

pub fn resolve_wallet_config(user_path: Option<&str>) -> Result<PathBuf> {
    if let Some(path) = user_path {
        return Ok(PathBuf::from(path));
    }

    if let Ok(env_path) = std::env::var("SUI_CONFIG") {
        return Ok(PathBuf::from(env_path));
    }

    Ok(default_wallet_config()?)
}
