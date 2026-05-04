---
title: "Agni Datta"
aliases: "Agni Datta"
linter-yaml-title-alias: "Agni Datta"
date created: 2026-05-04
date modified: 2026-07-21
---

<!-- @format -->

# [Agni Datta](https://agnidatta.com)

Rust-generated academic website with a Rust/Wasm browser runtime and an optional privacy-preserving Cloudflare analytics edge.

## Local development

Install rustup and dprint at the version recorded in `.dprint-version`. The repository pins Rust 1.97.1 and declares its `wasm32-unknown-unknown`, Clippy, and rustfmt requirements in `rust-toolchain.toml`. No Node project or handwritten JavaScript is required.

```bash
rustup toolchain install 1.97.1 --profile minimal --component clippy,rustfmt --target wasm32-unknown-unknown
cargo site serve
cargo site serve --port 8001  # optional when 8000 is occupied
```

The command builds the static site and browser Wasm, watches relevant sources, and serves the last valid build at [http://localhost:8000](http://localhost:8000). Local analytics are disabled and need no Cloudflare credentials.

The complete workflow is Cargo-driven.

```bash
cargo site build
cargo site serve
cargo site check
cargo site format
cargo site analytics --days 30 --format table
cargo site analytics --days 30 --format csv
```

`cargo site format` applies rustfmt and pinned dprint plugins to Rust, Jinja-compatible HTML, CSS, TOML, Markdown, and GitHub Actions YAML. `cargo site check` verifies formatting before running Clippy, tests, source audits, and the production build.

The opt-in Rust WebDriver suite needs the local server and a driver listening at `WEBDRIVER_URL`.

```bash
WEBDRIVER_URL=http://localhost:4444 cargo test -p browser-tests --test webdriver -- --ignored --test-threads=1
```

## Repository structure

```text
crates/
├── sitegen/             static generation and atomic output replacement
├── webapp/              browser state, theme persistence, and SPA navigation
├── analytics-worker/    Cloudflare edge forwarding and D1 aggregation
└── browser-tests/       opt-in Rust WebDriver integration tests
xtask/                   Cargo development, audit, server, and report commands
content/                 typed TOML content
templates/
├── layouts/             complete document layouts
├── pages/               route templates
└── components/          reusable header and footer
styles/                  ordered tokens, foundation, layout, component, and responsive CSS
static/assets/           source images and PDFs
migrations/              D1 schema migrations
public/                  ignored atomic build output
```

The build keeps the public URLs for CSS, images, PDFs, and every existing page. The browser receives complete HTML documents for direct loads and non-Wasm fallback navigation.

## Theme persistence

Dark is the initial default. An explicit theme change writes only `theme=light` or `theme=dark` as a first-party, one-year, `SameSite=Lax` cookie. Production cookies are secure. A valid legacy `localStorage` value is migrated once and removed.

## Location analytics

The optional Cloudflare Worker counts only successful HTML page views. It retains daily aggregates keyed by a bounded route, country code, and region code, and deletes rows older than 90 days. It does not read or store IP addresses, detailed location, browser metadata, navigation sources, cookies, or visitor identifiers.

Private reports use Cloudflare's D1 API and require these environment variables. The command bounds all reports to 1–90 days and never prints credentials.

```text
CLOUDFLARE_API_TOKEN
CLOUDFLARE_ACCOUNT_ID
CLOUDFLARE_D1_DATABASE_ID
```

Create a least-privilege token that can query only the analytics D1 database. There is no public reporting endpoint.

## Deployment

The GitHub Pages workflow validates and publishes `public/`. Edge activation remains manual until the Cloudflare account is configured.

Before activating the Worker, create the D1 database, replace the placeholder database identifier in `wrangler.toml`, apply `migrations/0001_location_page_views.sql`, and confirm the `agnidatta.com/*` Worker route and DNS zone. The manual `Deploy analytics edge` workflow requires `CLOUDFLARE_ACCOUNT_ID`, `CLOUDFLARE_D1_DATABASE_ID`, and `CLOUDFLARE_DEPLOY_API_TOKEN` repository secrets. Scope the deployment token only to the required Worker script, D1 database, and zone route.

```bash
cargo install --locked worker-build --version 0.8.5
npx --yes wrangler@4.112.0 d1 migrations apply agni-datta-site-analytics --remote
npx --yes wrangler@4.112.0 deploy
```

Review applicable privacy obligations before production activation. The public disclosure is available at `/privacy/`.

## License

[MIT](LICENSE)
