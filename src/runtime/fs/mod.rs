use std::{collections::HashMap, fs::read_dir, path::PathBuf};

mod file;

pub use file::File;

use log::*;

pub type FileHandle = (i16, File);

#[derive(Debug, Default)]
pub struct FsModule {
    files: HashMap<i16, File>,
    sources: HashMap<i16, PathBuf>,
    fs_root: PathBuf,
}

impl FsModule {
    pub fn new(fs_root: &str) -> Self {
        Self {
            files: HashMap::new(),
            sources: HashMap::new(),
            fs_root: fs_root.into(),
        }
    }

    pub fn scan_files(&mut self) {
        match read_dir(&self.fs_root) {
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

    pub fn make_file(&mut self) -> Option<FileHandle> {
        for x in 100..=999 {
            if !self.sources.contains_key(&x) {
                let mut f = self.fs_root.clone();
                f.push(x.to_string());
                self.sources.insert(x, f);
                return Some((x, File::default()));
            }
        }
        None
    }

    pub fn grab_file(&mut self, id: i16) -> Option<FileHandle> {
        self.files.remove_entry(&id)
    }

    pub fn return_file(&mut self, fh: FileHandle) {
        self.files.insert(fh.0, fh.1);
    }

    pub fn wipe_file(&mut self, id: i16) {
        self.sources.remove(&id);
    }
}
