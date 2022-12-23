# What still has to be done

## General
- [ ] Split up into multiple crates within a [cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
- [ ] Set up CI with clippy and rustfmt
- [ ] Publish initial crate versions
- [ ] Get help with stack design
- [ ] Create Contribution Guidelines

## Library
- [x] Parse Ankiding-Syntax out of markdown
- [x] Create HTML from Markdown
- [ ] Allow File Traversal for multiple markdown files with [`walkdir`](https://github.com/BurntSushi/walkdir)
- [ ] Embed markdown into `apkg` package
- [ ] Create Unit-Tests with [`cargo-nexttest`](https://nexte.st/)
- [ ] Add Logging with [`tracing`](https://docs.rs/tracing/latest/tracing/)

## CLI
- [x] Create CLI Interface with clap
- [ ] Print to the CLI the cards found and the file currently processed
- [ ] Allow for both file and directory input
- [ ] Provide terminal output with [`termcolor`](https://docs.rs/termcolor/latest/termcolor/)
- [ ] Publish Dockerfile

## CI
- [ ] Design CI-Workflow for Github
- [ ] Desgin CI

## Web-UI
- [ ] Choose tooling
- [ ] Figure out how to do it :D
- [ ] Create Docker-Compose

## Documentation
- [ ] Write formal report
- [ ] Create informal book with [`mdBook`](https://rust-lang.github.io/mdBook/)
