use std::{
    convert::TryFrom,
    fs::File,
    io,
    path::{Path, PathBuf},
};

use crate::{fs_ext, post::Post};

#[derive(Debug)]
pub enum BlogFile {
    Post(PathBuf, Post),
    Other(PathBuf, File),
}

impl BlogFile {
    pub fn simulate_move<P: AsRef<Path>>(self, out_dir: P) -> io::Result<Self> {
        let out_dir = out_dir.as_ref();
        match self {
            BlogFile::Post(path, inner) => {
                let new_p = fs_ext::simulate_move(path.as_ref(), out_dir)?;
                Ok(BlogFile::Post(new_p, inner))
            }
            BlogFile::Other(path, inner) => {
                let new_p = fs_ext::simulate_move(path.as_ref(), out_dir)?;
                Ok(BlogFile::Other(new_p, inner))
            }
        }
    }
}

impl TryFrom<&Path> for BlogFile {
    type Error = io::Error;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let owned = path.to_path_buf();
        let file = File::open(path)?;
        match path.extension() {
            Some(ftype) => match ftype.to_str() {
                Some("md") | Some("markdown") => Ok(Self::Post(owned, Post::read_from(file)?)),
                Some(_) | None => Ok(Self::Other(owned, file)),
            },
            None => Ok(Self::Other(owned, file)),
        }
    }
}
