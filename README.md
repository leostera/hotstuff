<h1 align="center"> 🔥 hotstuff </h1>

<p align="center"> A composable, incremental, no-nonsense static site generator </p>

<div align="center">
  <a href="https://www.patreon.com/AbstractMachines">
    <img alt="Become a Patron" src="https://c5.patreon.com/external/logo/become_a_patron_button.png" width="150px" />
  </a>
</div>

hotstuff is **no-nonsense** -- it makes almost no assumptions about how you
structure or write your content.

hotstuff is **composable** -- touch a few files in your existing folder structure
and it'll just work.

hotstuff is **incremental** -- it aggressively caches your project so you can use
it on thousands of files.

Honestly the best part is that you are free to drop your files however the hell
you feel like. Seriously, just put a bunch of markdown files and create an
empty `site` file near them. You're done.

It has 3 features:

1. it preserves your project layout
2. it is crazy fast
3. it has a tiny live-reloading server

That is it. You can write your content in Markdown and it will be compiled to
HTML following CommonMark. You can write your content in HTML and it will not
be touched.

## Getting Started

If you put a `hotstuff-project` file on the root of your project, hotstuff will look
throughout your whole project for `site` files.

`site` files simply tell `hotstuff` that this particular folder should be compiled
into a website.

So if you have your posts in the following structure:

```sh
my/website λ tree
.
├── pages
│   ├── First-post.md
│   └── Some-other-post.md
└── sections
    ├── about.md
    ├── hire-me.md
    └── projects.md
```

You just need to `touch` a few files:

```sh
my/website λ touch hotstuff-project
my/website λ touch pages/site sections/site
```

And you can run `hotstuff` to compile the website using the same tree structure
under a `_public` folder:

```sh
my/website λ hotstuff build
01:19:09 INFO :: Building project...
01:21:55 INFO :: Built 9 artifacts in 6ms
01:19:09 INFO :: Done in 7ms

my/website  λ tree
.
├── _public
│   ├── pages
│   │   ├── First-post.html
│   │   └── Some-other-post.html
│   └── sections
│       ├── about.html
│       ├── hire-me.html
│       └── projects.html
├── hotstuff-project
├── pages
│   ├── First-post.md
│   ├── Some-other-post.md
│   └── site
└── sections
    ├── about.md
    ├── hire-me.md
    ├── projects.md
    └── site
```

Which you can readily serve however you feel like. Upload to S3, Now, GCS,
Github pages, or wherever really.

When in doubt, check out the `examples` folder. All of the features will be
showcased there.

### Templating

You'll quickly notice that the bare compilation from Markdown to HTML doesn't
quite fit all use-cases. To alleviate this `hotstuff` lets you specify in your
`site` file a template file to be used for all the Markdown files within that
specific site.

Say you wanted to wrap all of the pages from the example above in a common
markup: add a `<meta charset="utf-8">` to all of them. You'd write a template
file:

```html
<html>
  <head>
    <meta charset="utf-8">
  </head>
  <body>
    {| document |}
  </body>
</html>
```

And in your `site` file you'd point to it:

```lisp
(template "path/to/template.html")
```

Voila! That's all it takes to get the templating up and running.

### Assets

To copy assets (any supporting file to your site) you can use the `(assets
...)` rule:

```lisp
(assets
  style.css
  logo.svg
  bg_music.midi)
```

And they will be automatically copied from their location, relative to the
`site` file.

You can also use the shorthand `.` instead of listing your assets to have all
the files in the folder copied over. This is not recursive.

## Credits

hotstuff is inspired by:

* the `cactus` static site generator
* the `dune` build system, with its composability
* the `bazel` build system, with its aggressive incremental compilation techniques
