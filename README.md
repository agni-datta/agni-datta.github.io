<!-- @format -->

# [Agni Datta](https://agnidatta.com)

PhD student in theoretical computer science at the University of Edinburgh.

Static site generator in Rust — [Tera](https://keats.github.io/tera/) templates, TOML data files, a small [WebAssembly](https://webassembly.org/) module for theme toggling and nav highlighting.

## Structure

```text
.
├── content/
│   ├── site.toml
│   ├── publications.toml
│   └── resources.toml
├── templates/
│   ├── base.html
│   ├── index.html
│   ├── publications.html
│   ├── resources.html
│   └── 404.html
├── sitegen/src/main.rs
├── wasm/src/lib.rs
├── static/assets/
│   ├── css/site.css
│   └── img/
├── scripts/build.sh
└── public/              # generated output
```

## Stack

- **Rust** — site generator and WASM module
- **Tera** — Jinja2-style templating
- **pulldown-cmark** — Markdown rendering
- **wasm-bindgen / web-sys** — Rust↔JS interop
- **MathJax 4** — LaTeX rendering (CDN)
- **GitHub Actions** — build and deploy

## Requirements

- [Rust](https://rustup.rs/) (stable)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

## Build

```bash
bash scripts/build.sh
```

Runs `sitegen` and compiles the WASM module, placing output in `public/`.

```bash
cd public && python3 -m http.server 8000
```

## Content

All content lives in `content/`. Edit `site.toml`, `publications.toml`, or `resources.toml`, then rebuild.

## Deploy

Push to `main` → [GitHub Actions](.github/workflows/deploy.yml) builds and deploys `public/` to `agnidatta.com`.

## License

[MIT](LICENSE)
