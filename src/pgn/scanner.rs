use std::path::{Path, PathBuf};

use walkdir::WalkDir;

/// Recursively scan `root` and return all `*.pgn` file paths found.
pub fn scan_pgn_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_type().is_file()
                && entry
                    .path()
                    .extension()
                    .map(|ext| ext.eq_ignore_ascii_case("pgn"))
                    .unwrap_or(false)
        })
        .map(|entry| entry.path().to_path_buf())
        .collect()
}
