# `blagger`
[![Crates.io](https://img.shields.io/crates/v/blagger)](https://crates.io/crates/blagger) ![MIT OR Apache licensed](https://img.shields.io/crates/l/blagger)

fer yer blag

`blagger` is the world's dumbest static site generator, rivaled only by that pipeline of Bash and Python scripts everyone thinks about making once.


## Usage
```
USAGE:
    blagger [FLAGS] [OPTIONS] --out-dir <out-dir>

FLAGS:
    -h, --help       Prints help information
    -a               Include hidden files
    -V, --version    Prints version information

OPTIONS:
    -I, --ignored-files <ignored-files>...         List of files to ignore [default: ]
    -i, --in-dir <in-dir>                          Input directory [default: .]
    -o, --out-dir <out-dir>                        Output directory
        --tag-pages-dir <tag-pages-dir>
            Directory relative to `${out-dir}` that tag pages will be rendered to [default: tags]

        --tag-template-html <tag-template-html>    Template for tag pages. If not given, tag pages are not generated
    -t, --template-html <template-html>            Path to template file for Markdown posts
```

### Templating
Everything in `in-dir` **except** Markdown files (files with an extension of `md` or `markdown`) and files with a path Rust considers equal to `template-html` is copied as-is into `out-dir`. This is your CSS, JavaScript, images, hand-written HTML (you psycho), whatever. The directory structure is preserved - `in/css/what/why.css` becomes `out/css/what/why.css`.

Markdown files are processed as follows:
1. The post is pre-processed and rendered.
   1. The file is spit at the first occurrence of `::===::\n` (that is, `::===::` and a newline.)
      * The first string split off is the "front matter". This is TOML with the following schema:
        * `title`: Required. A string.
        * `tags`: Required. A (possibly empty) list of strings.
        * `subtitle`: Optional string.
        * `date`: Optional TOML date-time.
      * The second string split off is the Markdown content of the post.
   2. The template file given by `template-html` is rendered. The front matter is provided to the template as an object called `front`, and the markdown content of the post as `md_content`.
      * In the template, a formatter named `markdown` is available for rendering Markdown strings.
2. The rendered file is written to `out-dir` using the same directory-preserving scheme as other files, with the extension changed to `html` so browsers know what the heck is going on.

The template file is never emitted to `out-dir`.
## To-do list
- [x] [Tag navigation pages](https://github.com/c-x-berger/blagger/blob/master/TAG_TEMPLATES.md)
- [ ] RSS feed generation
- [ ] Multiple templates available
- [ ] Sitemap?
