/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */
use crate::config::CynthiaConfClone;
use crate::ServerContext;
use std::path::PathBuf;
use tokio::sync::MutexGuard;

fn get_lifetime(pr: FilePriority, config_clone: CynthiaConfClone) -> u64 {
    let normal_cache_lifetime = config_clone.cache.lifetimes.assets;
    let step = 4;
    match pr {
        FilePriority::Permanent => 0,
        FilePriority::High => {
            let multiply_by = step;
            normal_cache_lifetime.checked_mul(multiply_by).unwrap_or(
                normal_cache_lifetime
                    .checked_rem(1)
                    .unwrap_or(multiply_by)
                    .checked_mul(multiply_by)
                    .unwrap_or(8),
            )
        }
        FilePriority::Normal => normal_cache_lifetime,
        FilePriority::Low => {
            // If the assets cache lifetime is default (1600), the time it'll live in the cache is 400 miliseconds

            let divide_by = step;
            // If the normal cache lifetime is not divisible by 2, first remove 1
            normal_cache_lifetime.checked_div(2).unwrap_or(
                normal_cache_lifetime
                    .checked_rem(1)
                    .unwrap_or(divide_by)
                    .checked_div(divide_by)
                    .unwrap_or(1),
            )
        }
        FilePriority::Once => 1,
        FilePriority::Custom(lifetime) => lifetime,
    }
}

/// File priority
/// Determines how long a file will be kept in cache
/// The cache lifetime is determined by the assets cache lifetime in the configuration file.
///
/// # Variants
/// - `Permanent` - File is kept in cache until the server is stopped.
/// - `High` - If the assets cache lifetime is default (1600), the time it'll live in the cache is 6400 miliseconds
/// - `Normal` - If the assets cache lifetime is default (1600), the time it'll live in the cache is 1600 miliseconds
/// - `Low` - If the assets cache lifetime is default (1600), the time it'll live in the cache is 400 miliseconds
/// - `Once` - File is kept in cache for one request, then removed. Formally it's kept in cache for 1 milisecond.
/// - `Custom(u64)` - Custom lifetime. The value is in miliseconds.
/// ## Performance notes
/// `Permanent`, `Once` and `Custom` are not affected by the assets cache lifetime. These are special cases, and should be used with caution.
///
#[allow(unused)]
pub(super) enum FilePriority {
    /// File is kept in cache until the server is stopped.
    Permanent,
    /// If the assets cache lifetime is default (1600), the time it'll live in the cache is 6400 miliseconds
    High,
    /// If the assets cache lifetime is default (1600), the time it'll live in the cache is 1600 miliseconds
    Normal,
    /// If the assets cache lifetime is default (1600), the time it'll live in the cache is 400 miliseconds
    Low,
    /// File is kept in cache for one request, then removed. Formally it's kept in cache for 1 milisecond.
    Once,
    /// Custom lifetime. The value is in miliseconds.
    Custom(u64),
}

pub(crate) fn fs_get(
    mut ctx: MutexGuard<ServerContext>,
    path: PathBuf,
    priority: FilePriority,
) -> Result<Vec<u8>, String> {
    let cttl = get_lifetime(priority, ctx.config.clone());
    let file_cache_id = format!("fs:{}", path.to_string_lossy());
    let file_cache = ctx.get_cache(&file_cache_id, cttl);
    // Check if cache hit
    if let Some(cache) = file_cache {
        return Ok(cache.0.to_vec());
    }
    // Cache miss
    let file = match std::fs::read(path) {
        Ok(f) => f,
        Err(e) => return Err(format!("{e}")),
    };
    match ctx.store_cache(&file_cache_id, &file, cttl) {
        Ok(f) => f,
        Err(e) => return Err(e.to_string()),
    };
    Ok(file)
}
