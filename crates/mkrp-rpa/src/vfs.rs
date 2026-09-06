use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

use crate::archive::RpaArchive;
use crate::error::RpaError;

pub struct Vfs {
    search_dirs: Vec<PathBuf>,
    archives: Vec<RpaArchive>,
    file_cache: HashMap<String, Vec<u8>>,
}

impl Vfs {
    pub fn new() -> Self {
        Self {
            search_dirs: Vec::new(),
            archives: Vec::new(),
            file_cache: HashMap::new(),
        }
    }

    /// Add a directory to the VFS search path. Files on disk take highest priority.
    pub fn mount_directory<P: AsRef<Path>>(&mut self, path: P) {
        let p = path.as_ref().to_path_buf();
        if p.is_dir() {
            info!("VFS mounted directory: {:?}", p);
            self.search_dirs.push(p);
        }
    }

    /// Mount an RPA archive into the VFS.
    pub fn mount_archive<P: AsRef<Path>>(&mut self, path: P) -> Result<(), RpaError> {
        let archive = RpaArchive::open(path)?;
        self.archives.push(archive);
        Ok(())
    }

    /// Automatically scan a game directory for all .rpa archives and mount them.
    pub fn mount_game_directory<P: AsRef<Path>>(&mut self, game_dir: P) -> Result<(), RpaError> {
        let p = game_dir.as_ref();
        if !p.is_dir() {
            return Ok(());
        }

        self.mount_directory(p);

        // Ren'Py loads archives in alphabetical order, or by specific priority
        let mut rpa_files = Vec::new();
        if let Ok(entries) = fs::read_dir(p) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext.eq_ignore_ascii_case("rpa") {
                            rpa_files.push(path);
                        }
                    }
                }
            }
        }

        rpa_files.sort();
        for rpa in rpa_files {
            match self.mount_archive(&rpa) {
                Ok(_) => info!("VFS loaded archive: {:?}", rpa.file_name().unwrap_or_default()),
                Err(e) => warn!("Failed to load RPA archive {:?}: {}", rpa, e),
            }
        }

        Ok(())
    }

    pub fn exists(&self, path: &str) -> bool {
        let normalized = path.replace('\\', "/");

        // 1. Check physical directories
        for dir in &self.search_dirs {
            let full_path = dir.join(&normalized);
            if full_path.is_file() {
                return true;
            }
        }

        // 2. Check archives
        for archive in &self.archives {
            if archive.contains(&normalized) {
                return true;
            }
        }

        false
    }

    pub fn read(&mut self, path: &str) -> Result<Vec<u8>, RpaError> {
        let normalized = path.replace('\\', "/");

        // 1. Check cache
        if let Some(data) = self.file_cache.get(&normalized) {
            return Ok(data.clone());
        }

        // 2. Check loose files in mounted directories
        for dir in &self.search_dirs {
            let full_path = dir.join(&normalized);
            if full_path.is_file() {
                return fs::read(&full_path).map_err(RpaError::Io);
            }
        }

        // 3. Check archives (in reverse order or priority order)
        for archive in self.archives.iter_mut().rev() {
            if archive.contains(&normalized) {
                return archive.read(&normalized);
            }
        }

        Err(RpaError::FileNotFound(normalized))
    }

    pub fn list_all_files(&self) -> Vec<String> {
        let mut files = Vec::new();

        // From directories
        for dir in &self.search_dirs {
            if let Ok(entries) = walkdir::WalkDir::new(dir).into_iter().collect::<Result<Vec<_>, _>>() {
                for entry in entries {
                    if entry.file_type().is_file() {
                        if let Ok(rel) = entry.path().strip_prefix(dir) {
                            let rel_str = rel.to_string_lossy().replace('\\', "/");
                            files.push(rel_str);
                        }
                    }
                }
            }
        }

        // From archives
        for archive in &self.archives {
            for name in archive.list_files() {
                files.push(name.to_string());
            }
        }

        files.sort();
        files.dedup();
        files
    }
}

impl Default for Vfs {
    fn default() -> Self {
        Self::new()
    }
}
