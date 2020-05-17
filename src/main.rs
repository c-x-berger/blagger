use std::{
    convert::TryFrom,
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
    #[structopt(short, long)]
    in_dir: PathBuf,
    /// Output directory.
    #[structopt(short, long)]
    out_dir: PathBuf,
    /// Path to template file for Markdown posts
    ///
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
    template_html: PathBuf,
}

fn main() -> io::Result<()> {
    let mut opts = Options::from_args();
    opts.in_dir = opts.in_dir.canonicalize()?;
    opts.template_html = opts.template_html.canonicalize()?;

    println!("Hello, world!");
    let mut template = String::new();
    BufReader::new(File::open(&opts.template_html)?).read_to_string(&mut template)?;
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
        .filter(|e| !(e.path() == opts.template_html))
        .map(|e| -> io::Result<_> {
            let path = e.path();
            let file = BlogFile::try_from(path.as_ref())?;
            let relpath = path.strip_prefix(&opts.in_dir).unwrap();
            Ok(file.change_path(opts.out_dir.join(relpath)))
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
