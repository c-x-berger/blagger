use std::{
    convert::TryFrom,
    fs::File,
    io,
    path::{Path, PathBuf},
};

use crate::post::Post;

#[derive(Debug)]
pub enum BlogFile {
    Post(PathBuf, Post),
    Other(PathBuf, File),
}

impl BlogFile {
    pub fn change_path(self, new: PathBuf) -> Self {
        match self {
            BlogFile::Post(_, inner) => (BlogFile::Post(new, inner)),
            BlogFile::Other(_, inner) => (BlogFile::Other(new, inner)),
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
