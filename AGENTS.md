<!-- @format -->

# Website rules

- Never underline links in any state. Do not imitate underlines with borders, shadows, or background images on text links. Use color and visible focus outlines.
- Preserve Albert Sans for body text, Space Mono for monospace text, and the Nocturne editorial design: darkened Nord for dark mode, sepia with Nord blue accents for light mode, circular icon controls, and separate page URLs.
- Collect no visitor data in website code. The only persistent preference is the light/dark theme cookie. Do not add analytics, tracking, browser storage, third-party embeds, or a collection backend. Load Albert Sans and Space Mono from Google Fonts, as requested by the author; do not bundle local font files. Google Fonts is the only permitted external subresource provider.
- Keep the Privacy Note short and accurate, including the hosting provider's separate logging.
- Keep CSS ownership clear: tokens, foundation, layout, components, then responsive rules. Edit the owning rule rather than appending a competing override layer.
- Use the shared 2.5pt corner radius as the default throughout the site. Preserve the circular icon controls and portrait.
- Use native browser navigation. Rust/Wasm handles the theme, mobile menu, email contact, and explicit citation-copy actions. Clipboard access is write-only and must follow a user click.
- Keep the contact address encoded in `person.email_token`. Show `person.email_display` with `[at]` and `[dot]`, and decode the token only when the visitor clicks that text. Never render the decoded address or a persistent `mailto:` link. This is reversible obfuscation.
- List only papers available online, with a link to the paper. Omit private submissions and work in preparation.
- Keep publication BibTeX entries in the single `static/assets/bib/references.bib` file. Match papers by their verified citation keys; do not create separate per-paper bibliography files.
- Run `cargo site check` for changes to the runtime or build tooling. Check appearance and interaction in both themes at desktop and mobile widths for CSS or template changes.
