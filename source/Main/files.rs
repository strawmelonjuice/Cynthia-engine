/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use log::{debug, trace};
use normalize_path::NormalizePath;

use crate::ServerContext;

pub(super) type CynthiaCache = Vec<CynthiaCacheObject>;
#[derive(Debug, Clone)]
pub(super) struct CynthiaCacheObject {
    id: String,
    content: Vec<u8>,
    timestamp: (u64, u64),
}
#[derive(Debug, Clone)]
pub(crate) struct CynthiaCacheExtraction(pub(crate) Vec<u8>, #[allow(dead_code)] pub(crate) u64);
impl ServerContext {
    pub(crate) fn store_cache(&mut self, id: &str, contents: &[u8], max_age: u64) {
        self.evaluate_cache();
        let cache = CynthiaCacheObject {
            id: id.to_string(),
            content: Vec::from(contents),
            timestamp: (
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    + max_age,
            ),
        };
        self.cache.push(cache);
    }
    pub(crate) fn get_cache(&mut self, id: &str, max_age: u64) -> Option<CynthiaCacheExtraction> {
        self.evaluate_cache();
        let object = self.cache.iter().find(|&x| x.id == id)?.clone();
        if max_age == 0
            || (SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                - object.timestamp.0)
                < max_age
                && object.timestamp.1
                    < SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
        {
            Some(CynthiaCacheExtraction(object.content, object.timestamp.0))
        } else {
            None
        }
    }
    fn evaluate_cache(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.cache
            .retain(|x| x.timestamp.1 > now || x.timestamp.1 == 0);
        debug!("Total cache size: {} bytes", self.estimate_cache_size());
        if !self.cache.is_empty() {
            trace!("Cache contents: {:?}", self.cache);
        } else {
            // Psst, just putting it here so it technically has a use to the compiler. It doesn't really do anything right now.
            self.clear_cache();
            trace!("Cache is empty.");
        }
    }
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
    pub fn estimate_cache_size(&self) -> usize {
        self.cache.iter().map(|x| x.content.len()).sum()
    }
}
#[allow(dead_code)]
fn cachefolder() -> PathBuf {
    let fl = tempfolder()
        .join(format!("{}", std::process::id()))
        .normalize();
    // logger(31, format!("Cache folder: {}", fl.display()));
    fs::create_dir_all(&fl).unwrap();
    fl
}
pub(crate) fn tempfolder() -> PathBuf {
    let fl = PathBuf::from("./.cynthiaTemp/")
        .join(format!("{}", std::process::id()))
        .normalize();
    // logger(31, format!("Cache folder: {}", fl.display()));
    fs::create_dir_all(&fl).unwrap();
    fl
}
