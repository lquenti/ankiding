# Ankiding - Markdown Notes to Anki Decks!

<sub>The documentation is still work in progress. Just open an issue in case you get stuck anywhere.</sub>

## Table of Contents

- [What exactly is this?](#what-exactly-is-this)
- [Installation](#installation)
- [Quickstart](#quickstart)
- [Automated CI Anki Deck Builds](#automated-ci-anki-deck-builds)
- [Contribution](#contribution)

## What exactly is this?

This tool allows to convert your Markdown Notes to Ankidecks! It also supports Images as well as LaTeX formulas. It has full mobile support, including mobile LaTeX support.

It is created in the same spirit as Jupyter Notebooks and literate programming in general. This means, you can just embed your flash cards within your notes and it will extract them

Furthermore, we have pre-built templates for both GitHub and GitLab CIs so that, on every commit to your Markdown knowledge base repository, a new `.apkg` package is being built. 

## Installation

For now, a full LaTeX suite is required in order to pre-render all LaTeX formula as the mobile apps do not support on demand formula rendering. On Debian/Ubuntu based systems:

```bash
# Everything else that has pdflatex and dvisvgm should probably suffice
sudo apt install texlive-full
```

Until we have automated builds, you also need the following build dependencies:
```bash
sudo apt install git build-essential pkg-config libssl-dev 
```
as well as a Rust compiler, which I recommend installing in userspace using [`rustup`](https://rustup.rs/).

Then, just clone and build it with
```bash
git clone https://github.com/lquenti/ankiding.git
cd ankiding
cargo build --release
cd ..
```

Your binary can then be found at `./ankiding/target/release/ankiding`.

## Quickstart

Create a folder that should represent your deck name, for example
```bash
mkdir programming
```

Then, for example, create the `./programming/python.anki.md` file with your casual notes, with flash cards interjected as you see fit:

```markdown
# Python

Python is a **great** programming language because
- One can easily start programming
- It has awesome tooling
- The Jupyter support is a gamechanger for interdisciplinary work
- It has libraries for everything

> Q: Why do people use Python a lot for scripting?
>
> A: Because it has a libraries for everything
> For example:
> - subprocess for process spawning
> - os for OS integration
> - json as a json parser

Python is also used a lot in different domains such as ML or backend programming

> Q: What is the most important Python web framework
> A: Django

> Q: What are some important Python micro web frameworks?
> A: Flask and bottlepy

> Q: With which Python library would one use for modern APIs?
> A: FastAPI
```

You can also link other local or remote files as images and split it into multiple `.anki.md` files. All files with other extensions just get ignored.

Once you are done, just run ankiding with `ankiding ./programming`.

Lastly, import the `output.apkg` into your Anki and you are done!

## Automated CI Anki Deck Builds

We understand that
- Many Anki users are non technical
- Having a full LaTeX instalation is a big hastle

Furthermore, Anki decks are notoriously awful to collaboratively work on while Markdown with its simplicity is perfect for any kind of versioning. Thus, we have created fully automated CI workflows **for both GitHub and GitLab**:

1. Clone template ([GitHub](https://github.com/lquenti/ankiding-ci-github), [GitLab](https://gitlab.gwdg.de/lars.quentin/ankiding-ci-gitlab))
2. Add `.anki.md` files into the `./assets` folder
3. Commit
4. Download created Artifacts

## Contribution

I am happy for any kind of help, but for now I am still experimenting with everything. So while I am happy for any kind of help, for big changes/ideas please reach out to me first to make sure that it fits the broader vision.

Once everything is documented and formally specified and a bit more stable I am happy to change this :)
