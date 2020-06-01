use std::{collections::HashSet, fmt::Write, io};

use pulldown_cmark::{html, Options, Parser};
use serde::Serialize;
use serde_json::Value;
use tinytemplate::{error::Result as TemplateResult, TinyTemplate};

use crate::post::Post;

const TEMPLATE_ID: &'static str = "tpl";
const ALL_OPTIONS: Options = Options::all();

#[derive(Serialize)]
pub struct CompiledPost {
    post: Post,
    path: String,
}

impl CompiledPost {
    pub fn new(post: Post, path: String) -> Self {
        Self { post, path }
    }

    pub fn post(&self) -> &Post {
        &self.post
    }
}

pub struct PostCompiler<'a> {
    tt: TinyTemplate<'a>,
    posts: Vec<CompiledPost>,
    tags: HashSet<String>,
}

impl<'a> PostCompiler<'a> {
    pub fn new(template: &'a str) -> Self {
        let mut tt = TinyTemplate::new();
        tt.add_template(TEMPLATE_ID, template).unwrap();
        tt.add_formatter("markdown", Self::template_md);
        tt.add_formatter("commasep", Self::commasep);

        Self {
            tt,
            posts: vec![],
            tags: HashSet::new(),
        }
    }

    fn parse(text: &str, md_opts: Options) -> String {
        let parser = Parser::new_ext(text, md_opts);
        let mut html = String::new();
        html::push_html(&mut html, parser);
        html
    }

    fn template_md(value: &Value, output: &mut String) -> TemplateResult<()> {
        match value {
            Value::String(s) => {
                output.push_str(&Self::parse(s, ALL_OPTIONS));
                Ok(())
            }
            Value::Number(n) => {
                write!(output, "{}", n)?;
                Ok(())
            }
            Value::Bool(b) => {
                write!(output, "{}", b)?;
                Ok(())
            }
            Value::Null => Ok(()),
            _ => Err(tinytemplate::error::Error::GenericError {
                msg: "Expected a printable value but found array or object.".to_string(),
            }),
        }
    }

    fn commasep(value: &Value, output: &mut String) -> TemplateResult<()> {
        match value {
            Value::Array(v) => {
                let formatted = v.iter().map(|json| {
                    let mut out = String::new();
                    (tinytemplate::format(json, &mut out), out)
                });
                let mut strings = Vec::new();
                for s in formatted {
                    match s {
                        (Ok(_), st) => strings.push(st),
                        (e @ Err(_), _) => return e,
                    }
                }
                output.push_str(&strings.join(", "));
                Ok(())
            }
            _ => Err(tinytemplate::error::Error::GenericError {
                msg: "Expected an array, got something else".to_string(),
            }),
        }
    }

    /// Parses `post` into HTML and returns the entire resulting page. `deployed_url` must be the
    /// URL of the page as is will appear in the final site, and is used for the tag system.
    /// Omitting it will cause `post` to not appear on tag list pages.
    pub fn parse_post(&mut self, post: Post, deployed_url: Option<String>) -> io::Result<String> {
        let ret = self
            .tt
            .render(TEMPLATE_ID, &post)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e));
        match ret {
            Ok(rendered) => {
                for tag in post.frontmatter().tags() {
                    self.tags.insert(tag.clone());
                }
                match deployed_url {
                    Some(url) => self.posts.push(CompiledPost::new(post, url)),
                    None => (),
                }
                Ok(rendered)
            }
            Err(e) => Err(e),
        }
    }

    pub fn tags(&self) -> &HashSet<String> {
        &self.tags
    }

    pub fn tagged_as(&self, tag: &str) -> Vec<&CompiledPost> {
        self.posts
            .iter()
            .filter(|p| p.post().frontmatter().tags().contains(&tag.to_owned()))
            .collect()
    }
}
