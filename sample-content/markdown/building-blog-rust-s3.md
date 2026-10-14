---
title: "Building a Blog with Rust and S3"
slug: "building-blog-rust-s3"
date: "2024-01-20"
author: "Jane Developer"
summary: "A deep dive into the architecture of Papaya CMS and how it leverages Rust and S3 for a modern blogging platform"
tags: ["rust", "architecture", "s3", "axum", "tera"]
cover_image: "https://images.unsplash.com/photo-1555066931-4365d14bab8c?w=1200&h=600&fit=crop"
---

# Building a Blog with Rust and S3

When we set out to build Papaya CMS, we had a clear vision: a **lightweight, fast, and flexible** blogging platform that doesn't require a database.

## Architecture Overview

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Client    │────▶│  Axum HTTP  │────▶│  In-Memory  │
│  (Browser)  │     │   Server    │     │    Cache    │
└─────────────┘     └─────────────┘     └─────────────┘
                           │                    │
                           ▼                    ▼
                    ┌─────────────┐     ┌─────────────┐
                    │    S3       │     │   Tera      │
                    │  (Content)  │     │  Templates  │
                    └─────────────┘     └─────────────┘
```

## The Stack

| Component | Technology | Reason |
|-----------|------------|--------|
| Web Framework | **Axum** | Ergonomic, performant, type-safe |
| Templating | **Tera** | Jinja2-like, familiar syntax |
| Markdown | **pulldown-cmark** | Fast, GFM-compliant |
| Storage | **AWS S3** | Scalable, durable, CDN-ready |
| Cache | **Arc<RwLock<>>** | Zero-copy reads, thread-safe |
| Config | **config-rs** | Layered, env-override support |
| Logging | **tracing** | Structured, JSON output |

## Content Pipeline

### 1. Markdown with Frontmatter

```markdown
---
title: "My Post"
slug: "my-post"
date: "2024-01-15"
tags: ["rust", "web"]
cover_image: "https://bucket.s3.amazonaws.com/cover.jpg"
---

# Content here...
```

### 2. Server Boot Sequence

```rust
// 1. Load configuration
let config = Config::load()?;

// 2. Initialize S3 client
let s3 = S3Client::new(&config).await?;

// 3. Initialize template engine
let templates = TemplateEngine::new(&args).await?;

// 4. Create cache
let cache = Cache::new();

// 5. Hydrate cache from S3
cache.refresh(&s3, &config).await?;
```

### 3. Request Flow

```
GET /posts/my-post
       │
       ▼
Cache lookup by slug (O(1) via HashMap)
       │
       ▼
S3 fetch markdown/my-post.md
       │
       ▼
Parse frontmatter + body
       │
       ▼
Markdown → HTML (pulldown-cmark)
       │
       ▼
Tera render post.html
       │
       ▼
Return HTML response
```

## Performance Characteristics

- **Cache reads**: Zero-allocation, lock-free for readers
- **Cache writes**: Single writer, blocks readers briefly
- **S3 calls**: Only on cache miss or refresh
- **Template rendering**: Pre-compiled Tera templates
- **Memory**: ~1KB per post in cache

## Custom Themes

Themes are loaded at container startup from a Git repository:

```bash
docker run \
  -e PAPAYA_TEMPLATE_REPO=https://github.com/user/themes \
  -e PAPAYA_TEMPLATE_NAME=minimal \
  papaya-cms
```

Template structure:
```
themes-repo/
└── templates/
    └── minimal/
        ├── home.html
        └── post.html
```

## Deployment

### Docker (Recommended)

```dockerfile
# Multi-stage build for minimal image
FROM rust:1.82-slim as builder
# ... build steps ...

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/papaya-backend /usr/local/bin/
ENTRYPOINT ["docker-entrypoint.sh"]
```

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `S3__ENDPOINT` | Yes | - | S3 endpoint URL |
| `S3__BUCKET` | Yes | - | Bucket name |
| `S3__ACCESS_KEY` | Yes | - | Access key ID |
| `S3__SECRET_KEY` | Yes | - | Secret access key |
| `API__REFRESH_TOKEN` | Yes | - | Cache refresh token |
| `PAPAYA_TEMPLATE_REPO` | No | papaya-cms | Template repo URL |
| `PAPAYA_TEMPLATE_NAME` | No | default | Template folder name |

## Future Enhancements

- [ ] Search functionality
- [ ] RSS/Atom feeds
- [ ] Webhook notifications
- [ ] Multi-language support
- [ ] Plugin system

---

*Built with ❤️ using Rust and Papaya CMS*