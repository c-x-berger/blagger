use std::{io, io::Read};

use serde::{Deserialize, Serialize};
use toml::value::Datetime;

const MATTER_SPLITTER: &'static str = "::===::\n";

#[derive(Serialize, Deserialize, Debug)]
pub struct FrontMatter {
    title: String,
    tags: Vec<String>,
    subtitle: Option<String>,
    description: Option<String>,
    date: Option<Datetime>,
}

impl FrontMatter {
    pub fn new(
        title: String,
        tags: Vec<String>,
        subtitle: Option<String>,
        description: Option<String>,
        date: Option<Datetime>,
    ) -> Self {
        Self {
            title,
            tags,
            subtitle,
            description,
            date,
        }
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }
}

#[derive(Debug, Serialize)]
pub struct Post {
    front: FrontMatter,
    md_content: String,
}

impl Post {
    pub fn new(front: FrontMatter, md_content: String) -> Self {
        Self { front, md_content }
    }

    pub fn read_from(reader: impl Read) -> io::Result<Self> {
        let mut reader = io::BufReader::new(reader);
        let mut text = String::new();
        reader.read_to_string(&mut text)?;
        let parts: Vec<&str> = text.splitn(2, MATTER_SPLITTER).collect();
        if parts.len() != 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "could not split into exactly two parts",
            ));
        }

        Ok(Post::new(toml::from_str(parts[0])?, String::from(parts[1])))
    }

    pub fn frontmatter(&self) -> &FrontMatter {
        &self.front
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_valid_content() -> io::Result<()> {
        let input = r#"title = 'Test Post Please Ignore'
subtitle = 'for real, ignore it this time'
tags = ['what', 'are', 'you', 'doing']
date = 2020-05-16 21:38:05+00:00
::===::
This is a post!"#;
        let post = Post::read_from(input.as_bytes())?;
        assert_eq!(
            post.frontmatter().title,
            String::from("Test Post Please Ignore")
        );
        assert_eq!(post.md_content, String::from("This is a post!"));
        assert_eq!(post.frontmatter().tags().len(), 4);
        Ok(())
    }
}
