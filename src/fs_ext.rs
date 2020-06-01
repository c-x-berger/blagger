use std::{fs, fs::DirEntry, io, io::ErrorKind, path::Path};

/// Ensure the parent of `path` exists, creating it recursively if need be.
pub fn ensure_parent<P: AsRef<Path>>(path: P) -> io::Result<()> {
    match path.as_ref().parent() {
        Some(p) => fs::create_dir_all(p),
        None => Ok(()),
    }
}

pub fn ensure_directory<P: AsRef<Path>>(path: P) -> io::Result<()> {
    match fs::create_dir_all(path.as_ref()) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
            if path.as_ref().is_dir() {
                Ok(())
            } else {
                Err(io::Error::new(
                    ErrorKind::InvalidInput,
                    "path exists but is not a dir",
                ))
            }
        }
        Err(e) => Err(e),
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

pub fn is_hidden<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().ancestors().any(|c| {
        c.file_name().is_some() && c.file_name().unwrap().to_string_lossy().starts_with(".")
    })
}
