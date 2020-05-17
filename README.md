# `blagger`
![Crates.io](https://img.shields.io/crates/l/blagger/0.1.0)

fer yer blag

`blagger` is the world's dumbest static site generator, rivaled only by that pipeline of Bash and Python scripts everyone thinks about making once.


## Usage
```
USAGE:
    blagger --in-dir <in-dir> --out-dir <out-dir> --template-html <template-html>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --in-dir <in-dir>                  Input directory
    -o, --out-dir <out-dir>                Output directory
    -t, --template-html <template-html>    Path to template file for Markdown posts.
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
- [ ] Multiple templates available
- [ ] Tag navigation pages
- [ ] Sitemap
