Proof-of-concept of extracting content from strapi and parsing them with remark plugins.

## Fetch strapi content

With nested populating, we can get all pages and their nested content for a certain volume in one go.

Then loop through the chunks for each page create markdown files.

```rust
cargo run
```

Look into `/output`

Some notable differences from the current workflow

- use `.md` instead of `.mdx`

- `

- `order` is added to frontmatter, which is determined by the order of content in strapi

- no section wrapping, that will be handled by the remark plugin. This keeps markdown clean.

## Parse markdown

In practice, this will be done byvelite. This is just to show that with custom remark plugins, we can parse them into the format Next.js wants.

```bash
cd srcts
pnpm build
```

Look into `test.html`
