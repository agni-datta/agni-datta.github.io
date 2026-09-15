<!-- @format -->

# [Agni Datta](https://agnidatta.com)

An academic website generated in Rust. Every page is a complete HTML document; a small Rust/Wasm runtime handles the theme switch, mobile menu, email button, and citation-copy buttons.

The Nocturne design uses Albert Sans for body text and Space Mono for code, loaded through one Google Fonts stylesheet with `display=swap`. The palette is darkened Nord in dark mode and sepia with Nord blue accents in light mode. Links are never underlined. Home, Publications, Notes, References, Miscellany, and Privacy Note have separate URLs.

## Local development

Install rustup and the dprint version in `.dprint-version`. The repository pins Rust 1.97.1, rustfmt, Clippy, and the Wasm target in `rust-toolchain.toml`.

```bash
rustup toolchain install 1.97.1 --profile minimal --component clippy,rustfmt --target wasm32-unknown-unknown
cargo site serve
cargo site serve --port 8001
```

The server watches source files and serves the last valid build at [localhost:8000](http://localhost:8000). Refresh after editing content, templates, or CSS. Restart the command after changing the Rust generator or build tooling.

```bash
cargo site build
cargo site check
cargo site format
```

`check` runs formatting checks, native and browser-Wasm Clippy with warnings denied, Rust tests, source audits, and the production build. The generated output is replaced only after a successful build and audit. `format` uses rustfmt and pinned dprint plugins.

The optional Rust WebDriver tests need a running site and local driver:

```bash
WEBDRIVER_URL=http://localhost:4444 cargo test -p browser-tests --test webdriver -- --ignored --test-threads=1
```

## Source layout

| Path                    | Purpose                                       |
| ----------------------- | --------------------------------------------- |
| `content/`              | Typed TOML content                            |
| `templates/`            | Layout, route pages, shared header and footer |
| `styles/tokens.css`     | Palettes, typography and spacing variables    |
| `styles/foundation.css` | Element defaults and link/focus rules         |
| `styles/layout.css`     | Page grids and containers                     |
| `styles/components.css` | Nocturne components                           |
| `styles/responsive.css` | Viewport, motion and print rules              |
| `static/assets/`        | Images, PDFs and BibTeX files                 |
| `crates/sitegen/`       | Static page generator                         |
| `crates/webapp/`        | Theme, navigation, email and citation actions |
| `crates/browser-tests/` | Optional WebDriver integration tests          |
| `xtask/`                | Cargo commands, local server and build audits |
| `public/`               | Ignored generated output                      |

`sitegen::STYLE_MODULES` defines the CSS order for bundling, hashing, and audits. Keep each base component rule in one place. Do not add theme-specific layout overrides or handwritten JavaScript/TypeScript; the build produces the Wasm loader.

The generator's `PAGES` list defines page output, canonical URLs, active navigation, and the sitemap. Home and Publications share `templates/components/paper.html`. Build audits live in `xtask/src/audit.rs`; `cargo site` is the single command entry point.

## Contact

Store the contact address as Base64 in `person.email_token` in `content/site.toml`, and its readable form with `[at]` and `[dot]` in `person.email_display`. The displayed text opens the visitor's email app on click; the decoded address is never inserted into page text or a persistent link. With JavaScript disabled, the readable form remains visible. This is reversible obfuscation, not protection against scraping, and earlier Git commits can still contain the original address.

## Citations

List only papers available online in `content/publications.toml`, with a link to each paper. Omit private submissions and work in preparation.

Keep all publication citations in `static/assets/bib/references.bib`. A paper's optional `citation` in `content/publications.toml` needs only its verified `key` and a `version` label. The generator reads the shared bibliography once and selects each paper's entry by its exact key for both the homepage and Publications page. Missing keys, duplicate keys and unclosed entries fail the build.

Add self-contained, brace-delimited entries to this file, with literal field values rather than external string macros or cross-references. The entry index preserves the original BibTeX text; it is not a full BibTeX validator. The Publications page offers the complete file through **Download bibliography (.bib)**. Individual papers retain expandable entries and Copy BibTeX buttons. The disclosures and download work without JavaScript; copying writes only the displayed entry after a click and reports whether it succeeded.

The current `EPRINT:CamDat24` record comes from the [official CryptoBib export](https://github.com/cryptobib/export/blob/master/crypto.bib), checked on 15 September 2026. It cites the 2024 preprint. The page separately lists the paper's ASIACRYPT 2026 venue; replace its entry in `references.bib` and its citation key when the official proceedings record becomes available.

## Privacy

The website has no analytics or collection backend. Google Fonts is its only external subresource provider; downloading the fonts shares the visitor's IP address and browser information with Google. The stylesheet request omits credentials and referrer information. The site's only persistent preference is a first-party `theme=light` or `theme=dark` cookie, written after an explicit switch. It lasts one year, uses `SameSite=Lax`, and is `Secure` on HTTPS. No browser storage or navigation history is recorded by the runtime; the browser handles normal page navigation.

The public [Privacy Note](https://agnidatta.com/privacy/) explains Google Fonts requests and GitHub Pages' separate security logging. Removing collection code locally does not change any previously configured external hosting services.

## Deployment

The GitHub Pages workflow reads the pinned Rust and dprint versions, checks the repository, and publishes `public/` when `main` is pushed. Only source files belong in Git; `public/` and `target/` are generated and ignored. Run `cargo site check` before pushing.

## License

Site code: [MIT](LICENSE). [Albert Sans](https://fonts.google.com/specimen/Albert+Sans) and [Space Mono](https://fonts.google.com/specimen/Space+Mono) are served by Google Fonts; no font files are bundled in this repository. The shared `--sans` and `--mono` tokens select the two families.
