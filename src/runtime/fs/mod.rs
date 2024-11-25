use std::{collections::HashMap, fs::read_dir, path::PathBuf};

mod file;

pub use file::File;
use log::*;

#[derive(Debug, Default)]
pub struct FsModule {
    files: HashMap<i16, File>,
    sources: HashMap<i16, PathBuf>,
}

impl FsModule {
    pub fn scan_files(&mut self, fs_root: &PathBuf) {
        match read_dir(fs_root) {
            Ok(results) => results
                .filter_map(|r| match r {
                    Ok(p) => {
                        if p.path().is_file() {
                            match p.metadata() {
                                Ok(meta) => {
                                    if meta.len() > i16::MAX as u64 {
                                        warn!("[WARN][fs] file too large");
                                        None
                                    } else {
                                        Some(p.path())
                                    }
                                }
                                Err(e) => {
                                    error!("[ERROR][fs] {}", e);
                                    None
                                }
                            }
                        } else {
                            None
                        }
                    }
                    Err(e) => {
                        error!("[ERROR][fs] {}", e);
                        None
                    }
                })
                .for_each(|p| {
                    let idx = *self.sources.keys().max().unwrap_or(&0);
                    self.sources.insert(idx, p);
                }),
            Err(e) => {
                error!("[ERROR][fs] {}", e);
            }
        }
    }
}
