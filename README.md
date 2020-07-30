<h1 align="center"> 🔥 hotstuff </h1>

<p align="center"> A composable, incremental, no-nonsense static site generator </p>

<div align="center">
  <a align="center" href="https://www.patreon.com/AbstractMachines">
    <img alt="Become a Patron" src="https://c5.patreon.com/external/logo/become_a_patron_button.png" width="150px" />
  </a>
</div>

Yes, we know of `{some other tool}`. No, it wasn't what we needed.

hotstuff is **no-nonsense** &mdash; it makes almost no assumptions about how
you structure or write your content. There's also no plugins whatsoever.

hotstuff is **composable** &mdash; touch a few files in your existing folder
structure and it'll just work. Near zero-cost, and your content is always fully
portable.

hotstuff is **incremental** &mdash;- it aggressively caches your project so you
can use it on hundreds of thousands of files.

## Getting Started

If you put a `hotstuff-project` file on the root of your project, hotstuff will
look throughout your whole project for `site` files.

`site` files tell `hotstuff` that this particular folder should be compiled
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

And you can run `hotstuff build` to compile the website using the same tree
structure under a `_public` folder:

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

hotstuff is inspired by prior art:

* the `cactus` static site generator
* the `dune` build system, with its composability
* the `bazel` build system, with its aggressive incremental compilation
  techniques
