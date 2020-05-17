use std::{
    fs,
    fs::DirEntry,
    io,
    path::{Component, Path, PathBuf},
};

/// Ensure the parent of `path` exists, creating it recursively if need be.
pub fn ensure_parent<P: AsRef<Path>>(path: P) -> io::Result<()> {
    match path.as_ref().parent() {
        Some(p) => fs::create_dir_all(p),
        None => Ok(()),
    }
}

/// Recursively get all contents of `dir` if dir is a directory. Error otherwise or on I/O errors.
pub fn all_contents<P: AsRef<Path>>(dir: P) -> io::Result<Vec<DirEntry>> {
    if !dir.as_ref().is_dir() {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "not a directory",
        ))
    } else {
        let mut result = Vec::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                result.append(&mut all_contents(&path)?);
            } else {
                result.push(entry);
            }
        }
        Ok(result)
    }
}
