use std::{
    convert::TryFrom,
    fs::File,
    io,
    io::{BufReader, BufWriter, Read},
    path::PathBuf,
};

use serde_json::json;
use structopt::StructOpt;
use tinytemplate::TinyTemplate;

mod blogfile;
use blogfile::BlogFile;
mod compiler;
use compiler::{CompiledPost, PostCompiler};
mod fs_ext;
mod post;
use post::*;

#[derive(StructOpt)]
#[structopt(about = "fer yer blag")]
struct Options {
    /// Input directory.
    #[structopt(short, long, default_value = ".")]
    in_dir: PathBuf,
    /// Output directory.
    #[structopt(short, long)]
    out_dir: PathBuf,
    /// Path to template file for Markdown posts
    ///
    /// Default is `${in-dir}/template.html`.
    /// Assumed to contain a valid [`tinytemplate`] template. The following values/formatters will
    /// be available in rendering:
    ///
    /// - `md_content`: The raw markdown content of the post being rendered.
    /// - `front`: A `FrontMatter` object with the following fields:
    ///   - `title`: The post's title.
    ///   - `subtitile`: The post's subtitle, or `None`.
    ///   - `date`: The post's publication date, or `None`.
    ///   - `tags`: A possibly-empty array of tags (strings.)
    #[structopt(short, long, verbatim_doc_comment)]
    template_html: Option<PathBuf>,
    /// Template for tag pages. If not given, tag pages are not generated.
    #[structopt(long)]
    tag_html: Option<PathBuf>,
    /// Template for the special "all tags" tag page. If not given, defaults to tag_html.
    #[structopt(long)]
    tag_hub: Option<PathBuf>,
    /// Directory relative to `${out-dir}` that tag pages will be rendered to.
    #[structopt(long, default_value = "tags")]
    tag_pages_dir: PathBuf,
    /// List of files to ignore
    #[structopt(short = "I", long, default_value = "")]
    ignored_files: Vec<PathBuf>,
    /// Include hidden files.
    ///
    /// A file is "hidden" if it or any of its parents up to `${in-dir}` start with a `.`.
    #[structopt(short = "a")]
    read_hidden: bool,
}

fn main() -> io::Result<()> {
    let mut opts = Options::from_args();
    opts.in_dir = opts.in_dir.canonicalize()?;

    println!("Hello, world!");

    let mut template = String::new();
    let template_path = match opts.template_html.as_ref() {
        Some(p) => p.clone(),
        None => opts.in_dir.join("template.html"),
    };
    let template_path = template_path.canonicalize()?;
    BufReader::new(File::open(&template_path)?).read_to_string(&mut template)?;
    opts.ignored_files.push(template_path);
    let mut compiler = PostCompiler::new(&template);

    fs_ext::ensure_directory(&opts.out_dir)?;
    opts.out_dir = opts.out_dir.canonicalize()?;

    match opts.tag_html.as_ref() {
        Some(p) => {
            opts.tag_pages_dir = opts.out_dir.join(opts.tag_pages_dir);
            fs_ext::ensure_directory(&opts.tag_pages_dir)?;
            opts.tag_pages_dir = opts.tag_pages_dir.canonicalize()?;
            opts.ignored_files.push(p.clone());
            if opts.tag_hub.is_none() {
                opts.tag_hub = opts.tag_html.as_ref().cloned();
            } else {
                opts.ignored_files
                    .push(opts.tag_hub.as_ref().unwrap().clone());
            }
        }
        None => {}
    }

    let entries = fs_ext::all_contents(&opts.in_dir)?;
    let mapped_files = entries.iter().filter_map(|e| -> Option<BlogFile> {
        let path = e.path();
        let relpath = path.strip_prefix(&opts.in_dir).unwrap();
        let mut skip = path.starts_with(&opts.out_dir);
        skip = skip || (!opts.read_hidden && fs_ext::is_hidden(relpath));
        skip = skip
            || opts
                .ignored_files
                .iter()
                .any(|f| f.canonicalize().is_ok() && path == f.canonicalize().unwrap());
        if skip {
            None
        } else {
            let file = BlogFile::try_from(path.as_ref()).unwrap();
            Some(file.change_path(opts.out_dir.join(relpath)))
        }
    });

    for file in mapped_files {
        match file {
            BlogFile::Other(dest, mut file) => {
                fs_ext::ensure_parent(&dest)?;
                let mut output = File::create(&dest)?;
                io::copy(&mut file, &mut output)?;
            }
            BlogFile::Post(mut dest, inner) => {
                fs_ext::ensure_parent(&dest)?;
                dest.set_extension("html");
                let mut output = BufWriter::new(File::create(&dest)?);
                // compile post
                let rel_dest = dest.strip_prefix(&opts.out_dir).unwrap();
                let rel_dest = String::from(rel_dest.to_string_lossy());
                let parsed = compiler.parse_post(inner, Some(rel_dest))?;
                io::copy(&mut parsed.as_bytes(), &mut output)?;
            }
        }
    }

    match opts.tag_html {
        Some(path) => {
            let mut templater = TinyTemplate::new();
            let mut template = String::new();
            File::open(path)?.read_to_string(&mut template)?;
            templater.add_template("tag", &template).unwrap();

            let rel_out_dir = opts.tag_pages_dir.strip_prefix(&opts.out_dir).unwrap();
            let mut all_tags: Vec<CompiledPost> = vec![];
            for tag in compiler.tags() {
                let value = json!({"tag": tag, "posts": compiler.tagged_as(tag)});
                let rendered = templater.render("tag", &value).unwrap();
                let dest = opts.tag_pages_dir.join(format!("{}.html", tag));
                let mut output = File::create(&dest)?;
                io::copy(&mut rendered.as_bytes(), &mut output)?;

                let fakematter = FrontMatter::new(tag.clone(), vec![], None, None, None);
                let fakepost = Post::new(fakematter, String::new());
                all_tags.push(CompiledPost::new(
                    fakepost,
                    String::from(rel_out_dir.join(tag).to_string_lossy()),
                ));
            }

            match opts.tag_hub {
                Some(path) => {
                    let mut templa = String::new();
                    File::open(path)?.read_to_string(&mut templa)?;
                    templater.add_template("all-tags", &templa).unwrap();

                    let value = json!({"tag": "all", "posts": all_tags});
                    let rendered = templater.render("all-tags", &value).unwrap();
                    let dest = opts.tag_pages_dir.join("all.html");
                    let mut output = File::create(dest)?;
                    io::copy(&mut rendered.as_bytes(), &mut output)?;
                }
                None => (),
            }
        }
        None => (),
    }
    println!("Done!");
    Ok(())
}
