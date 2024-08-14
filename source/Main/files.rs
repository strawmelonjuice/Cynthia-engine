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
    pub(crate) fn store_cache(
        &mut self,
        id: &str,
        contents: &[u8],
        max_age: u64,
    ) -> Result<(), ()> {
        self.evaluate_cache();
        let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(x) => x.as_secs(),
            Err(_) => {
                return Err(());
            }
        };
        let cache = CynthiaCacheObject {
            id: id.to_string(),
            content: Vec::from(contents),
            timestamp: (now, now + max_age),
        };
        self.cache.push(cache);
        Ok(())
    }
    pub(crate) async fn store_cache_async(
        &mut self,
        id: &str,
        contents: &[u8],
        max_age: u64,
    ) -> Result<(), ()> {
        self.evaluate_cache();
        let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(x) => x.as_secs(),
            Err(_) => {
                return Err(());
            }
        };
        let cache = CynthiaCacheObject {
            id: id.to_string(),
            content: Vec::from(contents),
            timestamp: (now, now + max_age),
        };
        self.cache.push(cache);
        Ok(())
    }
    pub(crate) fn get_cache(&mut self, id: &str, max_age: u64) -> Option<CynthiaCacheExtraction> {
        self.evaluate_cache();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let object = self
            .cache
            .iter()
            .find(|&x| {
                trace!("Cache check: {} - {:#?}", id, x.id);
                x.id == id
            })?
            .clone();
        trace!("Cache hit: {}", id);
        if max_age == 0 || ((now - object.timestamp.0) < max_age) {
            Some(CynthiaCacheExtraction(object.content, object.timestamp.0))
        } else {
            trace!("Cache devaluate: {}", id);
            None
        }
    }
    pub(crate) fn evaluate_cache(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.cache
            .retain(|x| x.timestamp.1 > now || x.timestamp.1 == 0);
        debug!("Total cache size: {} bytes", self.estimate_cache_size());
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
