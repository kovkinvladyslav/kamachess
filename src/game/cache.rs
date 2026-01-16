use anyhow::{Context, Result};
use chess::Board;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tracing::{debug, warn};

const CACHE_DIR: &str = "images_cache";
const DEFAULT_CACHE_SIZE_MB: u64 = 100;
const EVICTION_TARGET_PERCENT: u64 = 80; // Evict to 80% of limit

/// Get cached image or create it using the provided render function.
/// Handles cache size management with LRU eviction.
pub fn get_or_create<F>(board: &Board, flip_board: bool, render_fn: F) -> Result<Vec<u8>>
where
    F: FnOnce() -> Result<Vec<u8>>,
{
    let cache_dir = PathBuf::from(CACHE_DIR);

    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;
    }

    let file_path = get_cache_path(board, flip_board);

    if file_path.exists() {
        match read_cached_image(&file_path) {
            Ok(bytes) => {
                debug!("Cache hit: {}", file_path.display());
                return Ok(bytes);
            }
            Err(e) => {
                warn!("Failed to read cached image: {}", e);
            }
        }
    }

    debug!("Cache miss: {}", file_path.display());
    let bytes = render_fn()?;

    if let Err(e) = check_and_evict_if_needed(&cache_dir) {
        warn!("Cache eviction failed: {}. Continuing anyway.", e);
    }

    if let Err(e) = fs::write(&file_path, &bytes) {
        warn!("Failed to cache image: {}", e);
    } else {
        debug!("Cached image: {}", file_path.display());
    }

    Ok(bytes)
}

/// Get the cache file path for a board position
fn get_cache_path(board: &Board, flip_board: bool) -> PathBuf {
    let fen = board.to_string();
    let flip_suffix = if flip_board { "_flipped" } else { "" };
    let safe_fen = fen.replace(['/', ' '], "_");
    PathBuf::from(CACHE_DIR).join(format!("{}{}.png", safe_fen, flip_suffix))
}

/// Read cached image from disk
fn read_cached_image(path: &Path) -> Result<Vec<u8>> {
    let mut file = fs::File::open(path).context("Failed to open cached image")?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Check cache size and evict LRU files if limit exceeded
fn check_and_evict_if_needed(cache_dir: &Path) -> Result<()> {
    let max_size_mb = get_cache_size_limit_mb();
    let max_size_bytes = max_size_mb * 1024 * 1024;

    let current_size = calculate_cache_size(cache_dir)?;

    if current_size > max_size_bytes {
        let target_size = (max_size_bytes * EVICTION_TARGET_PERCENT) / 100;
        debug!(
            "Cache size {}MB exceeds limit {}MB. Evicting to {}MB",
            current_size / 1024 / 1024,
            max_size_mb,
            target_size / 1024 / 1024
        );
        evict_lru_files(cache_dir, current_size, target_size)?;
    }

    Ok(())
}

/// Calculate total size of all cached PNG files
fn calculate_cache_size(cache_dir: &Path) -> Result<u64> {
    let mut total_size = 0u64;

    for entry in fs::read_dir(cache_dir).context("Failed to read cache directory")? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("png") {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
            }
        }
    }

    Ok(total_size)
}

/// Evict LRU files until target size is reached
fn evict_lru_files(cache_dir: &Path, current_size: u64, target_size: u64) -> Result<()> {
    // Collect all PNG files with their metadata
    let mut files: Vec<(PathBuf, u64, SystemTime)> = Vec::new();

    for entry in fs::read_dir(cache_dir).context("Failed to read cache directory")? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("png") {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(mtime) = metadata.modified() {
                    files.push((path, metadata.len(), mtime));
                }
            }
        }
    }

    // Sort by modification time (oldest first) for LRU eviction
    files.sort_by_key(|(_, _, mtime)| *mtime);

    let mut freed_size = 0u64;
    let mut evicted_count = 0;

    for (path, size, _) in files {
        if current_size - freed_size <= target_size {
            break;
        }

        match fs::remove_file(&path) {
            Ok(_) => {
                freed_size += size;
                evicted_count += 1;
                debug!("Evicted: {}", path.display());
            }
            Err(e) => {
                warn!("Failed to evict {}: {}", path.display(), e);
            }
        }
    }

    debug!(
        "Evicted {} files, freed {}MB",
        evicted_count,
        freed_size / 1024 / 1024
    );

    Ok(())
}

/// Get cache size limit from environment variable
fn get_cache_size_limit_mb() -> u64 {
    std::env::var("IMAGE_CACHE_SIZE_MB")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(DEFAULT_CACHE_SIZE_MB)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_cache_size_limit_default() {
        std::env::remove_var("IMAGE_CACHE_SIZE_MB");
        assert_eq!(get_cache_size_limit_mb(), DEFAULT_CACHE_SIZE_MB);
    }

    #[test]
    fn test_get_cache_size_limit_from_env() {
        std::env::set_var("IMAGE_CACHE_SIZE_MB", "200");
        assert_eq!(get_cache_size_limit_mb(), 200);
        std::env::remove_var("IMAGE_CACHE_SIZE_MB");
    }

    #[test]
    fn test_get_cache_size_limit_invalid_env() {
        std::env::set_var("IMAGE_CACHE_SIZE_MB", "invalid");
        assert_eq!(get_cache_size_limit_mb(), DEFAULT_CACHE_SIZE_MB);
        std::env::remove_var("IMAGE_CACHE_SIZE_MB");
    }
}
