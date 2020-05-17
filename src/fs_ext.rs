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

/// Return the result of "moving" the **relative** path `file` to the **directory** given by `out`.
///
/// To avoid bizzare behaviour, this function errors if any of the following are true:
///
/// - `file` is not a relative path
/// - `file` exists and is not a file
/// - `out` exists and is not a directory
///
/// Additionally, this function makes the assumptions that no bizarre symlinks have been used to
/// create directory loops, and that `file` is made of mostly "normal" components.
pub fn simulate_move<P: AsRef<Path>>(file: P, out: P) -> io::Result<PathBuf> {
    let file = file.as_ref();
    let out = out.as_ref();
    if !file.is_relative() || (file.exists() && !file.is_file()) || (out.exists() && !out.is_dir())
    {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "file or path is of wrong type",
        ))
    } else {
        // Stupid HACK: We straight-up discard anything weird in `file`.
        // Don't use this unless you are confident there won't be anything weird in there (such as
        // `..`, `.`, etc)
        let mut components: Vec<_> = file
            .components()
            .filter(|c| match c {
                Component::Normal(_) => true,
                _ => false,
            })
            .map(|c| c.as_os_str())
            .collect();
        components.remove(0);
        let mut ret = out.to_path_buf();
        ret.extend(components);
        Ok(ret)
    }
}
