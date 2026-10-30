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
  date_rfc3339: String,
  date_display: String,
  date_year: String,
  tags: Vec<String>,
  cover_image: Option<String>,
  photo: serde_json::Value  // free-form photography metadata
}
```

### post.html extras

- `newer_post` / `older_post` — adjacent entries (or null at either end)
- `post_number` / `post_total` — position in the archive, newest first
- `now_year` — current year for footers

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

## Aperture Theme

`templates/aperture/` is a light-only photographic archive built for large image
sets. The home page opens with an editorial cover — the newest frame tucked
under an oversized Bodoni wordmark — then turns vertical scroll into a
horizontal filmstrip of full-height photographs. Tiles are virtualized: only
the frames near the viewport exist in the DOM, off-screen images are released,
and they are requested again when you scroll back. A custom cursor, clip-path
reveals, and a thumbnail-to-full-frame page transition stay out of the way of
the photographs.

The post page is a full-bleed image (tall frames scroll at their natural
height), followed by a quiet title, the story, and a capture-details grid:
camera, lens, focal length, aperture, shutter, ISO, white balance, medium,
location, and date. Every field is optional and rendered from Markdown.

### Photography frontmatter

```markdown
---
title: "Still Water"
slug: "still-water"
date: "2026-09-18T07:12:00Z"
summary: "First light on a lake that refused to move."
author: "Aseem Savio"
cover_image: "http://127.0.0.1:9000/papaya/assets/images/still-water.jpg"
tags: ["Landscape", "Water"]
photo:
  camera: "Hasselblad X2D 100C"
  lens: "XCD 45mm f/3.5"
  aperture: "f/8"
  shutter: "1/250"
  iso: "64"
  focal_length: "45mm"
  white_balance: "Daylight · 5600K"
  film: "Digital · medium format"
  location: "Lake Bled, Slovenia"
  aspect: 1.4995       # width / height, used to size the filmstrip
  thumb: "http://127.0.0.1:9000/papaya/assets/images/still-water-thumb.jpg"
  color: "#9799a2"     # placeholder colour while the image loads
---

Body copy in Markdown…
```

`aspect`, `thumb`, and `color` are optional but recommended: `thumb` keeps the
home page light, `aspect` controls tile proportions before the image arrives,
and `color` prevents a white flash.

### Run the demo locally

The theme works with any S3-compatible endpoint. This checkout was seeded
against a local endpoint on `127.0.0.1:9000`:

```bash
# 1. Start a local S3-compatible server on port 9000
#    (moto is used for the checked-in demo; MinIO works the same way)
moto_server -H 127.0.0.1 -p 9000

# 2. Download the demo frames into .local-content/images/<pexels-id>.jpg
#    (these are intentionally git-ignored)

# 3. Create thumbnails, Markdown, and upload everything
python3 scripts/seed_aperture.py

# 4. Run Papaya with the theme
cargo run -- --template-dir ./templates --template-name aperture
```

Open <http://127.0.0.1:8080>.

The seed script uses `S3_ENDPOINT`, `S3_BUCKET`, `S3_ACCESS_KEY`, and
`S3_SECRET_KEY` (defaults match `config.toml`) and publishes `assets/*` for
anonymous reads so the browser can load photographs directly from S3.
`.local-content/` and `.minio-data/` are git-ignored; no photographs or
generated Markdown are committed.

## License

MIT
