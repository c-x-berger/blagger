use std::{
    convert::TryFrom,
    ffi::OsStr,
    fs,
    fs::File,
    io,
    io::{BufReader, BufWriter, Read},
    path::PathBuf,
};

use structopt::StructOpt;

mod blogfile;
mod compiler;
use compiler::PostCompiler;
mod fs_ext;
mod post;
use blogfile::BlogFile;

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
    /// List of files to ignore
    #[structopt(short = "I", long, default_value = "")]
    ignored_files: Vec<String>,
    #[structopt(short = "a")]
    read_hidden: bool,
}

fn main() -> io::Result<()> {
    let mut opts = Options::from_args();
    opts.in_dir = opts.in_dir.canonicalize()?;

    println!("Hello, world!");
    let mut template = String::new();
    let template_path = match &opts.template_html {
        Some(p) => p.clone(),
        None => opts.in_dir.join("template.html"),
    };
    BufReader::new(File::open(&template_path)?).read_to_string(&mut template)?;
    let mut compiler = PostCompiler::new(&template);
    if !opts.out_dir.exists() {
        fs::create_dir_all(&opts.out_dir)?;
    } else {
        if !opts.out_dir.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "output path was not a directory",
            ));
        }
    }
    opts.out_dir = opts.out_dir.canonicalize()?;
    let entries = fs_ext::all_contents(&opts.in_dir)?;
    let mapped_files = entries
        .iter()
        .filter_map(|e| -> Option<io::Result<BlogFile>> {
            let path = e.path();
            let relpath = path.strip_prefix(&opts.in_dir).unwrap();
            if path != template_path && !path.starts_with(&opts.out_dir) {
                if !opts.read_hidden
                    && relpath.ancestors().any(|c| {
                        c.file_name()
                            .unwrap_or(OsStr::new(""))
                            .to_string_lossy()
                            .starts_with(".")
                    })
                {
                    return None;
                }
                if !opts.ignored_files.iter().any(|f| relpath.ends_with(f)) {
                    let file = BlogFile::try_from(path.as_ref()).unwrap();
                    return Some(Ok(file.change_path(opts.out_dir.join(relpath))));
                }
                None
            } else {
                None
            }
        })
        .map(|r| r.unwrap());
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
    println!("Done!");
    Ok(())
}
