<!-- @format -->

# [Agni Datta's Homepage](https://agnidatta.com)

Personal academic homepage for Agni Datta, currently a PhD student in theoretical computer science at the University of Edinburgh.

## Overview

A static site generator written in Rust, using [Tera](https://keats.github.io/tera/) templates and a single TOML data file. A small [WebAssembly](https://webassembly.org/) module handles client-side theme toggling and navigation highlighting.

## Structure

```bash
.
├── content/
│   └── site.toml          # All site data: bio, publications, teaching, etc.
├── templates/             # Tera HTML templates
│   ├── base.html
│   ├── index.html
│   ├── publications.html
│   ├── resources.html
│   └── 404.html
├── sitegen/               # Rust crate: static site generator
│   └── src/main.rs
├── wasm/                  # Rust/WASM crate: theme toggle & nav
│   └── src/lib.rs
├── static/
│   └── assets/
│       ├── css/site.css
│       └── img/
├── scripts/
│   └── build.sh
└── public/                # Generated output (do not edit)
```

## Tech Stack

- **Rust** — site generator and WASM module
- **Tera** — Jinja2-style HTML templating
- **pulldown-cmark** — Markdown rendering
- **wasm-bindgen / web-sys** — Rust↔JS interop
- **MathJax 4** — LaTeX math rendering (CDN)
- **GitHub Actions** — build and deploy to GitHub Pages

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

## Building Locally

```bash
bash scripts/build.sh
```

This runs the Rust site generator (`sitegen`) and then compiles the WASM module via [wasm-pack](https://rustwasm.github.io/wasm-pack/), placing all output in `public/`.

To serve the generated site locally:

```bash
cd public && python3 -m http.server 8000
```

## Updating Content

All content lives in [content/site.toml](content/site.toml). Edit that file to update the bio, publications, teaching entries, talks, or service records — then rebuild.

## Deployment

Pushing to `main` triggers the [GitHub Actions workflow](.github/workflows/deploy.yml), which builds the site and deploys `public/` to GitHub Pages at the custom domain `agnidatta.com`.

## Acknowledgements

This template is based on the previous version of a homepage built by [Archisman Dutta](https://github.com/DeviousCilantro).

## License

[MIT](LICENSE)
