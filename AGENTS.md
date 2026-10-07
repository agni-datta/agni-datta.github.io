<!-- @format -->

# Website rules

- Never underline links in any state. Do not imitate underlines with borders, shadows, or background images on text links. Use color and visible focus outlines.
- Preserve Albert Sans for body text, Space Mono for monospace text, and the Nocturne editorial design: darkened Nord for dark mode, sepia with Nord blue accents for light mode, and circular icon controls.
- Collect no visitor data in website code. The only persistent preference is the light/dark theme cookie. Do not add analytics, tracking, browser storage, third-party embeds, or a collection backend. Load Albert Sans and Space Mono from Google Fonts, as requested by the author; do not bundle local font files. Google Fonts is the only permitted external subresource provider.
- Keep the Privacy Note short and accurate, including the hosting provider's separate logging.
- Keep one content file, section template, and section stylesheet per homepage section. `templates/pages/home.html` only composes sections; reusable entries belong in `templates/components/`. See the section map in `README.md`.
- Keep CSS ownership clear: tokens, foundation, layout, shared components, section styles, then responsive rules. `sitegen::STYLE_MODULES` defines the bundle order and audits reject unlisted CSS files. Edit the owning rule rather than appending a competing override layer.
- Use the shared 2.5pt corner radius as the default throughout the site. Preserve the circular icon controls and portrait.
- Keep all academic sections on the homepage. About stays visible; Publications uses a native disclosure that starts expanded. Other sections, reference categories, and reference topics use native disclosures that start folded. References always starts folded on load and when returning to the page, including fragment links. Do not reintroduce a table of contents or navigation menu.
- Use native browser navigation. Rust/Wasm handles the theme, publication filtering, fragment-link expansion, email contact, and explicit citation-copy actions in separate browser modules. Publication searches stay in the current page without storage or network requests. Clipboard access is write-only and must follow a user click.
- Keep the contact address encoded in `email_token` in `content/profile.toml`. Show `email_display` with `[at]` and `[dot]`, and decode the token only when the visitor clicks that text. Never render the decoded address or a persistent `mailto:` link. This is reversible obfuscation.
- List only papers available online, with a link to the paper. Omit private submissions and work in preparation.
- Generate the entire publication list from `static/assets/bib/references.bib`. Each entry needs a title, authors, year, and URL or DOI. Keep BibTeX citations from ePrint; add arXiv links through display overrides. Do not maintain a second paper list in TOML or create per-paper bibliography files. `content/publications.toml` holds only the heading, author homepage links, and optional display overrides keyed by verified citation keys.
- Preserve the exact original BibTeX text for copying and downloading. Keep publication authors in surname order in BibTeX; the page follows that order. Use that order for CryptoBib-style author abbreviations in citation keys.
- Run `cargo site check` for changes to the runtime or build tooling. Check appearance and interaction in both themes at desktop and mobile widths for CSS or template changes.
