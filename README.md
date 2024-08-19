Proof-of-concept of extracting content from strapi and parsing them with remark plugins.

## Refactoring on fetching strapi content

Goal:

- Get all page data in markdown files at build time, so the frontend can be completely decoupled from strapi during runtime. Importantly, constructed response items are now stored in the markdown frontmatter.

- Use markdown instead of mdx, this can prevent unexpected js injection and weird escape behavior. At run time, next.js will only transform certain tags into custom components.

```rust
cargo run <volume_id>
```

This generates one markdown document per page in the `output` folder. Example:

```markdown
title: 2. Program Structure
slug: 2-program-structure
order: 1
chunks:
- Introduction-203pt
- Expressions-and-Statements-775t
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

- since cri is in md, they are rendered as static elements instead of being inserted by `useEffect`

## Parse markdown

In practice, this part will be done by Next.js. This is just to show that with custom remark plugins, we can parse them into the correct html markup.

```bash
cd srcts
pnpm build
```

Look into `output-html`


## On the strapi side


### Required

- components:
   - new names:
     - `Sandbox` -> `i-sandbox-js`
     - `Blockquote` -> `i-blockquote`
     - `Info` -> `i-callout variant="info"`
     - `Image` -> `i-image`
     - `Accordion` -> `i-accordion` and `i-accordion-item`

  - multi-word props should be separated by hyphens, and not be camelCased.

    ```html
    <i-sandbox-js page-slug = "page" chunk-slug = "chunk">
        content
    </i-sandbox-js>
    ```

  - newlines should be inserted after and before component tags, i.e.
    ```html
    <i-callout variant="info">
    <!-- newline here -->
        content
    <!-- newline here -->
    </i-callout>
    ```

  - lists should be separated by new lines, i.e.
    ```markdown
    - item 1

    - item 2

    - item 3
    ```


### Optional

- Add a `description` field for page, mainly for accessibility and SEO purposes, can be AI generated.  Description will they will not be visible to regular readers, which means they won't be used as a hint for summary. Still, they should not be thorough, instead act more like a 2-4 sentence preface that motivates people to continue reading.



## Future work

- Generalized assignments: whether a page requires a summary, quiz or code exercises can be represented by a single `assignments` field

    ```
    ---
    assignments:
    - summary
    - quiz
    - exercises
    ---
    ```

    How do we include data for quiz and exercises? Potentially could do this

    ```yaml
    quiz:
    - question: How is the minus operator different in JavaScript?
      options:
        - text: "it can be both unary and binary"
        - correct: true
        - text: "it can only be unary"
        - correct: false

    exercises:
        - type: "javascript"
          prompt: "Write a function that adds two numbers"
          placeholder: "function add(a, b) {  }"
    ```

    But I am afraid this could lead to poor abstractions in which we find YAML is not expressive enough.


- To simplify parsing and chunk revealing, all h2 headings are treated as chunks. This is not ideal for "References" and "Exercises" chunks, they are typically the last chunk of a page and should be revealed automatically when the previous chunk is revealed. A quick fix is to add them as h3 headings at the end of the previous chunk. I think this is ok, but if we want to perfect this here are the required changes:

  - still treat references and exercises as h2 headings, but have a field indicating if it is a standalone chunk (which means they are blurred and revealed independently) or a auxiliary chunk (which means they are revealed when the previous chunk is revealed).

  - change relevant remark plugins to add attribute to the wrapper section element.

  - change `question-control.tsx` to inspect the attribute and reveal the chunk accordingly.
