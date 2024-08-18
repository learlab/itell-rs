Proof-of-concept of extracting content from strapi and parsing them with remark plugins.

## Fetch strapi content

Goal:

- Get all page data in markdown files at build time. So the frontend can be completely decoupled from strapi during runtime. Importantly, constructed response items are now stored in the markdown frontmatter.

- Use markdown instead of mdx, this can prevent unexpected js injection and weird escape behavior. At run time, next.js will only transform certain tags into custom components.

```rust
cargo run
```

This generates one markdown document per page in the `output` folder. Example:

```markdown
title: 2. Program Structure
slug: 2-program-structure
order: 1
assignments:
- summary
cri:
- question: What is the main substance of any JavaScript program according to the passage?
  answer: Creating values by applying operators to them.
  slug: Expressions-and-Statements-775t

## Introduction {#Introduction-203pt .sr-only}

In this chapter, we will start to do things that can actually be called _programming_. We will expand our command of the JavaScript language beyond the nouns and sentence fragments we’ve seen so far to the point where we can express meaningful prose.

## Expressions and Statements {#Expressions-and-Statements-775t}

In Chapter 1, we made values and applied operators to them to get new values. Creating values like this is the main substance of any JavaScript program. But that substance has to be framed in a larger structure to be useful. That’s what we’ll cover in this chapter.
```

Except for the goals mentioned above, other notable differences from the current workflow include

- use `${slug}.md` instead of `chapter-{number}.mdx`,

- `order` is added to frontmatter, which is determined by the order of content in strapi, this ensures that there is a single source of truth in page order.

- no section wrapping, which will be handled by the remark plugin instead. This keeps markdown clean.

Some details

- With nested populating, we can get all pages and their nested content for a certain volume in one go. Then loop through the chunks for each page create markdown files.

- We can publish this script as a homebrew binary (whether have it stay Rust or rewrite it in another language) and call it from github workflows. This will reduce publishing time.

## Parse markdown

In practice, this part will be done by Next.js. This is just to show that with custom remark plugins, we can parse them into the correct html markup.

```bash
cd srcts
pnpm build
```

Look into `test.html`


## On the strapi side

- components:
   - new names:
     - `Sandbox` -> `is-sandbox-js`
     - `Blockquote` -> `i-blockquote`
     - `Info` -> `i-callout variant="info"`
     - `Image` -> `i-image`
     - `Accordion` -> `i-accordion` and `i-accordion-item`

  - avoid using self-closing tags (even if there is no children)
