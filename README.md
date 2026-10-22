# Papaya CMS

A modern, high-performance blog platform built with Rust, Axum, Tera templates, and AWS S3 for content storage.

## Features

- **Server-side rendering** with Tera (Jinja2-like templates)
- **Content stored in S3** - Markdown files with YAML frontmatter
- **In-memory caching** - Fast metadata access with refresh endpoint
- **Mobile-responsive** - Tailwind CSS via CDN
- **Dockerized** - Template repository cloned at runtime
- **Custom themes** - Support for user-defined template repositories

## Architecture

```
├── backend/                 # Rust Axum server
│   ├── src/
│   │   ├── main.rs         # App routing, S3 client initialization
│   │   ├── cache.rs        # In-memory global state for metadata cache
│   │   ├── config.rs       # Configuration management
│   │   ├── s3.rs           # S3 client wrapper
│   │   ├── templates.rs    # Tera compilation and template loading
│   │   ├── error.rs        # Error handling
│   │   └── logging.rs      # Structured logging
│   └── Cargo.toml
├── templates/              # Isolated user-definable theme folders (cloned at runtime)
│   └── default/
│       ├── home.html       # Mobile-responsive home page
│       └── post.html       # Mobile-responsive single post layout
├── Dockerfile
├── docker-entrypoint.sh
├── docker-compose.yml
├── config.example.toml
└── Cargo.toml
```

## S3 Bucket Structure

```
your-bucket/
├── markdown/
│   ├── my-first-post.md
│   └── another-post.md
└── assets/
    ├── images/
    └── videos/
```

## Markdown Format

Each markdown file in `markdown/` must have YAML frontmatter:

```markdown
---
title: "My First Post"
slug: "my-first-post"
date: "2024-01-15"
author: "John Doe"
summary: "A brief summary for listings and SEO"
tags: ["rust", "web", "blog"]
cover_image: "https://your-bucket.s3.amazonaws.com/assets/images/cover.jpg"
---

# Your content here

Supports **markdown**, tables, code blocks, images, etc.
```

## Quick Start

### 1. Configuration

Copy `config.example.toml` to `config.toml` and fill in your values:

```bash
cp config.example.toml config.toml
```

Required environment variables (or use config.toml):
- `S3__ENDPOINT` - S3 endpoint URL
- `S3__REGION` - AWS region
- `S3__BUCKET` - Bucket name
- `S3__ACCESS_KEY` - Access key
- `S3__SECRET_KEY` - Secret key
- `API__REFRESH_TOKEN` - Secure token for cache refresh endpoint

### 2. Run with Docker Compose

```bash
docker-compose up --build
```

### 3. Run with Docker (Production)

```bash
docker build -t papaya-cms .
docker run -d \
  -p 8080:8080 \
  -e S3__ENDPOINT=https://s3.amazonaws.com \
  -e S3__REGION=us-east-1 \
  -e S3__BUCKET=your-bucket \
  -e S3__ACCESS_KEY=your-key \
  -e S3__SECRET_KEY=your-secret \
  -e API__REFRESH_TOKEN=your-token \
  papaya-cms
```

### 4. Custom Templates

```bash
docker run -d \
  -p 8080:8080 \
  -e PAPAYA_TEMPLATE_REPO=https://github.com/your-username/your-templates \
  -e PAPAYA_TEMPLATE_NAME=my-theme \
  ...other env vars...
  papaya-cms
```

Template repository structure:
```
your-templates-repo/
└── templates/
    └── my-theme/
        ├── home.html
        └── post.html
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/` | Home page - lists all posts |
| GET | `/posts/:slug` | Single post page |
| GET | `/health` | Health check |
| POST | `/api/refresh-cache` | Refresh metadata cache (requires Bearer token) |

## Cache Refresh

```bash
curl -X POST http://localhost:8080/api/refresh-cache \
  -H "Authorization: Bearer your-secure-random-token-here"
```

## Development

### Prerequisites

- Rust 1.82+
- Docker (for containerized deployment)

### Local Development

```bash
# Build
cargo build --release

# Run
cargo run --release
```

## Template Variables

### home.html
- `site_name` - Site name from config
- `site_description` - Site description from config
- `posts` - Array of post metadata objects

### post.html
- `site_name` - Site name from config
- `post` - Post metadata object
- `content` - Rendered HTML content (safe)

### Post Metadata Object
```rust
{
  title: String,
  slug: String,
  summary: Option<String>,
  author: Option<String>,
  date: DateTime<Utc>,
  tags: Vec<String>,
  cover_image: Option<String>
}
```

## Markdown Features

- Tables (GFM)
- Footnotes
- Strikethrough
- Task lists
- Smart punctuation
- Code blocks with syntax highlighting
- Blockquotes
- Images with captions (using figure/figcaption)
- Galleries (images with class="gallery")

## Otia Theme

`templates/otia/` ships a dark-only editorial theme inspired by the Otia studio
site: near-black canvas (`#050505`), ambient radial glow, film grain, Playfair
Display headings with Source Serif 4 body and JetBrains Mono code, thin
hairlines, and a single sage/mint/lime accent budget.

Run it locally:

```bash
./target/debug/papaya-backend --template-dir ./templates --template-name otia
```

### Component map

The theme's runtime (`partials/scripts.html`) upgrades plain markdown output:
framed code blocks with language label + copy button, wrapped tables, typed
callouts, task-list styling, heading anchors, footnote grouping, image
lightbox, carousels, and Mermaid diagrams (dark themed, rendered client-side;
Prism + Mermaid load via CDN with `defer`).

| Author this markdown | Renders as |
|---|---|
| `#`–`######` headings | Display type, auto anchor-linkable |
| `> [!NOTE]` `[!TIP]` `[!IMPORTANT]` `[!WARNING]` `[!CAUTION]` | Typed callouts (GitHub style) |
| Plain `>` blockquotes | Pull quote; `<cite>` styled |
| ```` ```lang ```` | Framed code block, Prism highlight, copy button |
| ```` ```mermaid ```` | Dark-themed Mermaid diagram |
| ```` ```carousel ```` (`src \| caption` per line) | Carousel (arrows, dots, counter, drag, autoplay) |
| `<div class="gallery">` with `<img>`s | Same carousel (legacy markup) |
| `![alt](src)` / `<figure><img><figcaption>` | Ringed image, lightbox on click (Esc closes) |
| GFM table | Hairline table, header tint, aligned columns |
| `[^1]` footnotes | Grouped "Notes" section at the end |
| `- [ ]` / `- [x]` | Selectable-style task list |
| `<dl>`, `<details>` | Styled definition list / disclosure |
| `~~strike~~`, `` `code` ``, `<kbd>`, `<mark>` | Inline marks |

Carousel slides intentionally skip the lightbox (they are navigable inline);
prose images feed it.

## License

MIT