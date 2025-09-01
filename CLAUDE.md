# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

iTELL's Rust SDK - a proof-of-concept tool for extracting content from Strapi CMS and converting it into markdown files with frontmatter for static site generation. The project fetches educational textbook content, validates vector embeddings, and generates markdown files that can be consumed by Next.js applications.

## Architecture

The project consists of three main components:

1. **Rust Core (`src/`)** - Main library and binary for fetching and processing CMS content
   - `cms/` module handles all Strapi CMS interactions and data models
   - `bin/fetch_volume.rs` - Main binary for content extraction
   - Core functionality: fetch volume data → collect pages → serialize to markdown → validate embeddings

2. **TypeScript Parser (`srcts/`)** - Optional markdown-to-HTML transformation using remark/rehype plugins
   - Demonstrates how the generated markdown can be processed into HTML
   - Uses custom remark plugins for educational content features

3. **Build System** - Shell script wrapper with predefined volume configurations

## Development Commands

### Building and Testing
```bash
# Build the release binary
make build
# or
cargo build --release

# Run tests with specific volume configurations
make test          # Test volume
make demo          # Demo volume  
make chevron       # Chevron textbook
make middlesex     # Middlesex textbook
# ... (see Makefile for all available volumes)

# Manual volume fetching
cargo run --bin fetch_volume <volume_id> [output_directory]
# or use the build script wrapper
./build.sh <volume_id> <target_directory>
```

### TypeScript Parser (Optional)
```bash
cd srcts
pnpm build  # Generates HTML in output-html/
```

## Configuration

The main binary requires environment variables for Supabase vector database access:
- `EMBEDDINGS_SUPABASE_URL` - Supabase project URL  
- `EMBEDDINGS_SUPABASE_API_KEY` - Supabase API key

Create a `.env` file in the project root with these variables.

## Key Features

- **Content Extraction**: Fetches nested page structures from Strapi CMS with constructed response items
- **Markdown Generation**: Creates clean markdown files with YAML frontmatter (no MDX)
- **Vector Validation**: Health check system that verifies all content chunks have corresponding embeddings in Supabase
- **Educational Components**: Handles special iTELL components like sandboxes, callouts, images, and accordions
- **Page Ordering**: Maintains content order from CMS as single source of truth

## Output Structure

Generated files in `output/` or specified directory:
- `volume.yaml` - Volume metadata (title, slug, description, free pages)
- `{page-slug}.md` - Individual page files with frontmatter containing chunks, assignments, and CRI data

The binary exits with code 0 on successful validation, 1 on validation failures (used in CI/CD workflows).