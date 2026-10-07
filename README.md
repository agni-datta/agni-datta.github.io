<!-- @format -->

# [Agni Datta](https://agnidatta.com)

An academic website generated in Rust. Every page is a complete HTML document; a small Rust/Wasm runtime handles the theme switch, publication filter, section expansion for fragment links, email button, and citation-copy buttons.

The Nocturne design uses Albert Sans for body text and Space Mono for code, loaded through one Google Fonts stylesheet with `display=swap`. The palette is darkened Nord in dark mode and sepia with Nord blue accents in light mode. Links are never underlined. Home and Privacy Note have separate URLs. The homepage lists all publications at `/#publications`, expository notes after Talks at `/#notes`, and reading references at `/#references`. About stays visible; Publications starts expanded, and Teaching, Service, Talks, Notes, and References start folded. Each section except About uses a collapsible heading with plus/minus indicators. Reference categories and topics also expand independently. Each note has a PDF link and an expandable Overview containing its description.

References starts folded on page load and when returning to the page, including direct `/#references` links. Visitors expand it through its heading.

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

| Path                    | Purpose                                             |
| ----------------------- | --------------------------------------------------- |
| `content/site.toml`     | Site title, description and base URL                |
| `content/`              | One typed TOML file per section                     |
| `templates/layouts/`    | HTML document shell                                 |
| `templates/pages/`      | Home composition, Privacy Note and 404              |
| `templates/sections/`   | One template per homepage section                   |
| `templates/components/` | Shared chrome, entries and profile icons            |
| `styles/tokens.css`     | Palettes, typography and spacing variables          |
| `styles/foundation.css` | Element defaults, prose and link/focus rules        |
| `styles/layout.css`     | Page grids and containers                           |
| `styles/components/`    | Chrome, controls, sections, entries and disclosures |
| `styles/sections/`      | One stylesheet per homepage section                 |
| `styles/responsive.css` | Viewport, motion and print rules                    |
| `static/assets/`        | Images, PDFs and the shared bibliography            |
| `crates/sitegen/`       | Content, rendering, assets and bibliography         |
| `crates/webapp/`        | Theme, search, sections, contact and citations      |
| `crates/browser-tests/` | Optional WebDriver integration tests                |
| `xtask/`                | Build, checks, server and audits                    |
| `public/`, `target/`    | Ignored generated output and build artifacts        |

Each homepage section has matching filenames across `content/`, `templates/sections/`, and `styles/sections/`:

| Section          | Filename stem  |
| ---------------- | -------------- |
| Profile          | `profile`      |
| About            | `about`        |
| Publications     | `publications` |
| Teaching         | `teaching`     |
| Service          | `service`      |
| Talks            | `talks`        |
| Expository notes | `notes`        |
| References       | `references`   |

`teaching.toml`, `service.toml`, `talks.toml`, and `notes.toml` use `[[entries]]`. Publications comes from the shared BibTeX file; `publications.toml` only configures its presentation. References has nested `[[groups]]`, `[[groups.topics]]`, and `[[groups.topics.items]]` matching its expandable categories and topics. Unknown content fields fail the build, so removed fields cannot silently remain unused.

`sitegen::STYLE_MODULES` in `crates/sitegen/src/assets.rs` defines the CSS order for bundling, hashing, and audits. Every stylesheet must appear there. Shared rules belong in `styles/components/`; section styles contain only section-specific rules. Do not add theme-specific layout overrides or handwritten JavaScript/TypeScript; the build produces the Wasm loader.

`crates/sitegen/src/lib.rs` coordinates generation. Its sibling modules handle typed content (`content.rs`), templates and routes (`render.rs`), assets (`assets.rs`), BibTeX parsing (`bibliography.rs`), publication cards (`publications.rs`), and the footer year (`calendar.rs`). The `PAGES` list in `render.rs` defines output routes, canonical URLs, and the sitemap. Homepage section templates use `components/paper.html`, `components/note.html`, and `components/reference-groups.html` for their entries. Section, BibTeX, and Overview disclosures use native browser controls.

`crates/webapp/src/browser/` separates theme, publications, contact, citations, and section actions; `mod.rs` registers events and dispatches clicks. Outside that directory, `theme.rs` contains the theme preference model and cookie rules, and `publications.rs` matches search terms; both can be tested without a browser. `xtask/src/main.rs` dispatches commands to `build.rs`, `checks.rs`, and `serve.rs`; `tools.rs` runs the pinned tools. `audit.rs` checks generated output, the CSS manifest, and every Rust browser module.

## Contact

Store the contact address as Base64 in `email_token` in `content/profile.toml`, and its readable form with `[at]` and `[dot]` in `email_display`. The displayed text opens the visitor's email app on click; the decoded address is never inserted into page text or a persistent link. With JavaScript disabled, the readable form remains visible. This is reversible obfuscation, not protection against scraping, and earlier Git commits can still contain the original address.

## Publications and citations

Add, edit, or remove papers in `static/assets/bib/references.bib`. The generator creates a publication card for every entry, including its title, authors, paper links, year, and BibTeX disclosure. The publication count updates automatically. No matching TOML entry is needed. During local development the server rebuilds when this file changes; refresh to see the result.

The publication search filters titles, coauthors, and venue labels as you type and shows the matching count. Multiple search terms must all match the same paper, regardless of case. Queries stay in the current page; the runtime neither stores nor sends them. Folding the section preserves the current search. Without the runtime, the search control stays hidden; section folding and BibTeX disclosures still work.

Each entry needs `title`, `author`, `year`, and either `url` or `doi`. Only include papers available online; omit private submissions and work in preparation. Authors follow the BibTeX order, which should be surname order for these papers. The build parser handles brace-protected titles, TeX accents, and both `First Last` and `Last, First` names. Original entry text remains unchanged in the disclosure and shared bibliography.

Venue labels come from `booktitle` or `journal`/`journaltitle`; ePrint entries default to **Preprint**. Paper links use **ePrint**, **PDF**, **Paper**, or **DOI** according to their fields. Cards sort by descending year, keeping bibliography order within a year. `content/publications.toml` contains the section heading, optional `[author_links]` keyed by display name, and optional `[overrides."CITATION:KEY"]` with `title`, `venue`, `year`, or an `arxiv` URL. The `arxiv` override adds an **arXiv** link while the BibTeX citation remains the ePrint record. Unknown authors display as plain text until you add a homepage link. Overrides affect the card, while the citation keeps its own title and year; an override never creates a paper that is absent from the bibliography.

Add self-contained, brace-delimited entries with literal field values rather than external string macros or cross-references. Missing required fields, duplicate keys or fields, and malformed entries fail the build with an explanation. Individual papers have their paper links beside a **BibTeX** disclosure, with the citation key visible only inside the expanded entry. The disclosures work without JavaScript; copying writes only the displayed entry after a click and reports whether it succeeded.

The current `EPRINT:CamDat24` record comes from the [official CryptoBib export](https://github.com/cryptobib/export/blob/master/crypto.bib), checked on 15 September 2026. It cites the 2024 preprint. The page separately lists the paper's ASIACRYPT 2026 venue; replace its entry in `references.bib` and its citation key when the official proceedings record becomes available.

The `EPRINT:BenDatYog26` entry comes from the paper's [official ePrint page](https://eprint.iacr.org/2026/2227), checked on 5 October 2026. We list its authors in surname order in both the homepage and bibliography. The paper is not yet in the CryptoBib export; its key uses that author order with the [CryptoBib labeling conventions](https://cryptobib.di.ens.fr/manual).

## Privacy

The website has no analytics or collection backend. Google Fonts is its only external subresource provider; downloading the fonts shares the visitor's IP address and browser information with Google. The stylesheet request omits credentials and referrer information. The site's only persistent preference is a first-party `theme=light` or `theme=dark` cookie, written after an explicit switch. It lasts one year, uses `SameSite=Lax`, and is `Secure` on HTTPS. No browser storage or navigation history is recorded by the runtime; the browser handles normal page navigation.

The public [Privacy Note](https://agnidatta.com/privacy/) explains Google Fonts requests and GitHub Pages' separate security logging. Removing collection code locally does not change any previously configured external hosting services.

## Deployment

The GitHub Pages workflow reads the pinned Rust and dprint versions, checks the repository, and publishes `public/` when `main` is pushed. Only source files belong in Git; `public/` and `target/` are generated and ignored. Run `cargo site check` before pushing.

## License

Site code: [MIT](LICENSE). [Albert Sans](https://fonts.google.com/specimen/Albert+Sans) and [Space Mono](https://fonts.google.com/specimen/Space+Mono) are served by Google Fonts; no font files are bundled in this repository. The shared `--sans` and `--mono` tokens select the two families.
