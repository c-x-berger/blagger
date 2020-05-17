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
struct Options {
    #[structopt(short, long)]
    in_dir: PathBuf,
    #[structopt(short, long)]
    out_dir: PathBuf,
    #[structopt(short, long)]
    template_html: PathBuf,
}

fn main() -> io::Result<()> {
    let mut opts = Options::from_args();

    println!("Hello, world!");
    let in_dir_str = match opts.in_dir.to_str() {
        Some(s) => String::from(s),
        None => opts.in_dir.to_string_lossy().into_owned(),
    };
    let out_dir_str = match opts.out_dir.to_str() {
        Some(s) => String::from(s),
        None => opts.out_dir.to_string_lossy().into_owned(),
    };
    let mut template = String::new();
    BufReader::new(File::open(&opts.template_html)?).read_to_string(&mut template)?;
    let mut compiler = PostCompiler::new(&template);
    print!("Checking if {} exists: ", out_dir_str);
    if !opts.out_dir.exists() {
        println!("it does not. Creating.");
        fs::create_dir_all(&opts.out_dir)?;
    } else {
        print!("it does.\nChecking that it is a directory: ");
        if !opts.out_dir.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "output path was not a directory",
            ));
        }
        println!("it is. Continuing with confidence.")
    }
    opts.out_dir = opts.out_dir.canonicalize()?;
    println!("Building file list of {}...", in_dir_str);
    let entries = fs_ext::all_contents(&opts.in_dir)?;
    let entries = entries.iter().filter(|e| !(e.path() == opts.template_html));
    let mapped_files = entries
        .map(|entry| -> io::Result<_> { Ok(BlogFile::try_from(entry.path().as_ref())?) })
        .map(|r| r.unwrap())
        .map(|f| f.simulate_move(&opts.out_dir).unwrap());
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
    Ok(())
}
