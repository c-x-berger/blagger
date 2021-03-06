# Tag Pages Templating

Tag pages are... annoying. They're annoying internally, and some of that annoyance has unfortunately been passed to the user.

The object passed to the tag template has the following fields:

* `tag` - the tag this page is being rendered for
* `posts` - an array of `(post, path)` pairs
  * `path` is the path the post has been rendered to, **relative** to the root. Using this usefully, in say an `a`, will probably require sticking a `/` on the front of it.
  * `post` has the same fields as the object passed to post rendering (indeed, it is the same object.)

Tag pages are rendered to `${tag-pages-dir}/${tag}.html`.

## The tag hub
The tag hub template gets the exact same values as a normal tag template, with the following notable differences:

* `tag` is always `all`
* The posts in the `posts` array are essentially empty. They will be titled after thier tag, and will have a `path` as expected (the path of that tag's tag page relative to root), but have basically no other content.

At time of writing, the tag hub is always rendered to `${tag-pages-dir}/all.html`, effectively disabling the use of the tag `all` for posts.
