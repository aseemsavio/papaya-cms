---
title: "Welcome to Papaya CMS"
slug: "welcome-to-papaya"
date: "2024-01-15"
author: "Papaya Team"
summary: "Introducing Papaya - a lightweight, fast blog platform built with Rust and S3"
tags: ["rust", "blog", "cms", "s3"]
cover_image: "https://images.unsplash.com/photo-1486312338219-ce68d2c6f44d?w=1200&h=600&fit=crop"
---

# Welcome to Papaya CMS 🍌

Papaya is a modern, high-performance blog platform built with **Rust**, **Axum**, **Tera templates**, and **AWS S3** for content storage.

## Why Papaya?

- **Fast** - Server-side rendering with in-memory caching
- **Simple** - Content stored as Markdown files in S3
- **Flexible** - Custom themes via Git repositories
- **Lightweight** - No database, no complex infrastructure
- **Mobile-first** - Responsive design out of the box

## Features

### Markdown Support

Papaya supports full GitHub-flavored Markdown including:

- **Tables** with alignment
- **Footnotes** [^1]
- **Strikethrough** ~~text~~
- **Task lists** [x] Done
- **Code blocks** with syntax highlighting

### Tables

| Feature | Status | Notes |
|---------|:------:|-------|
| Server-side rendering | ✅ | Tera templates |
| S3 content storage | ✅ | Markdown + assets |
| In-memory cache | ✅ | Thread-safe |
| Custom themes | ✅ | Git-based |
| Mobile responsive | ✅ | Tailwind CSS |

### Code Blocks

```rust
fn main() {
    println!("Hello, Papaya! 🍌");
    
    let posts = cache.get_posts().await;
    for post in posts {
        println!("{} - {}", post.date, post.title);
    }
}
```

```javascript
// JavaScript example
async function fetchPosts() {
    const response = await fetch('/api/posts');
    const posts = await response.json();
    return posts;
}
```

### Blockquotes

> Papaya makes blogging simple again. No databases, no complex CMS, just Markdown files in S3 rendered beautifully.

### Images & Galleries

Single image:

![Mountain landscape](https://images.unsplash.com/photo-1506905925346-21bda4d32df4?w=800&h=600&fit=crop)

Gallery:

<div class="gallery">
  <img src="https://images.unsplash.com/photo-1470071459604-3b5ec3a7fe05?w=400&h=300&fit=crop" alt="Forest">
  <img src="https://images.unsplash.com/photo-1441974231531-c6227db76b6e?w=400&h=300&fit=crop" alt="Ocean">
  <img src="https://images.unsplash.com/photo-1501785888041-af3ef285b470?w=400&h=300&fit=crop" alt="Mountains">
  <img src="https://images.unsplash.com/photo-1472214103451-9374bd1c798e?w=400&h=300&fit=crop" alt="Sunset">
</div>

### Lists

**Unordered:**
- Rust for performance
- Axum for web framework
- Tera for templating
- S3 for storage

**Ordered:**
1. Write Markdown
2. Push to S3
3. Refresh cache
4. View your blog

**Task list:**
- [x] Initialize project
- [x] Configure S3
- [x] Create templates
- [ ] Add comments
- [ ] Add search

---

## Getting Started

See the [README](../README.md) for full setup instructions.

[^1]: Footnotes work great for references and citations.