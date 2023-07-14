use std::{
    env,
    path::{Path, PathBuf},
};

use lazy_static::lazy_static;

use crate::LogResult;

lazy_static! {
    static ref TEMP_DIR: PathBuf = {
        let path = env::temp_dir().join("audio-tldr");
        std::fs::create_dir_all(&path).log_err("Failed to create temporary directory");
        path
    };
}

#[derive(Debug)]
pub struct TempFile {
    path: PathBuf,
}

impl TempFile {
    pub fn new<P: AsRef<Path>>(filename: P) -> Self {
        let path = TEMP_DIR.join(filename);
        log::trace!("New  tempfile: {path:?}");
        Self { path }
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        log::trace!("Drop tempfile: {:?}", self.path);
        std::fs::remove_file(&self.path).log_warn("Removing tempfile failed");
    }
}

impl AsRef<Path> for TempFile {
    fn as_ref(&self) -> &Path {
        AsRef::<Path>::as_ref(&self.path)
    }
}
