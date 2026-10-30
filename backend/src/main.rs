use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use clap::Parser;
use mime_guess::from_path;
use pulldown_cmark::{html, Options, Parser as MdParser};
use tera::Context;
use tokio::signal;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

mod cache;
mod config;
mod error;
mod logging;
mod s3;
mod templates;

use cache::Cache;
use config::Config;
use error::AppError;
use s3::S3ClientRef;
use templates::TemplateEngine;

#[derive(Clone)]
struct AppState {
    cache: Arc<Cache>,
    s3_client: S3ClientRef,
    templates: TemplateEngine,
    config: Arc<Config>,
}

#[derive(Parser, Debug)]
#[command(name = "papaya", about = "Papaya CMS - Lightweight blog platform")]
struct Args {
    #[arg(long, default_value = "https://github.com/aseemsavio/papaya-cms", env = "PAPAYA_TEMPLATE_REPO")]
    template_repo: String,

    #[arg(long, default_value = "default", env = "PAPAYA_TEMPLATE_NAME")]
    template_name: String,

    #[arg(long, default_value = "/templates", env = "PAPAYA_TEMPLATE_DIR")]
    template_dir: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    logging::init();

    let args = Args::parse();

    info!("Starting Papaya CMS");
    info!("Template repo: {}", args.template_repo);
    info!("Template name: {}", args.template_name);
    info!("Template dir: {}", args.template_dir);

    let config = Arc::new(Config::load()?);
    info!("Configuration loaded");

    let s3_client = S3ClientRef::new(s3::S3Client::new(&config).await?);
    info!("S3 client initialized");

    let templates = TemplateEngine::new(&args.template_dir, &args.template_repo, &args.template_name).await?;
    info!("Template engine initialized");

    let cache = Arc::new(Cache::new());
    info!("Cache initialized");

    if let Err(e) = cache.refresh(&s3_client, &config).await {
        warn!("Initial cache refresh failed: {}", e);
    } else {
        info!("Initial cache refresh completed");
    }

    let server_host = config.server.host.clone();
    let server_port = config.server.port;

    let state = AppState {
        cache,
        s3_client,
        templates,
        config,
    };

    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind((server_host.as_str(), server_port)).await?;
    info!("Server listening on {}:{}", server_host, server_port);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/", get(home_handler))
        .route("/posts/{slug}", get(post_handler))
        .route("/assets/{*path}", get(asset_proxy_handler))
        .route("/api/refresh-cache", post(refresh_cache_handler))
        .route("/health", get(health_handler))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn home_handler(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    let posts = state.cache.get_posts().await;
    let mut context = Context::new();
    context.insert("posts", &posts);
    context.insert("site_name", &state.config.site.name);
    context.insert("site_description", &state.config.site.description);
    let home_description = if state.config.site.home_description.is_empty() {
        &state.config.site.description
    } else {
        &state.config.site.home_description
    };
    context.insert("home_description", home_description);

    context.insert("base_url", &state.config.site.base_url);
    context.insert("now_year", &chrono::Utc::now().format("%Y").to_string());

    let html = state.templates.render("home.html", &context)?;
    Ok(Html(html))
}

async fn post_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Html<String>, AppError> {
    let post = state.cache.get_post_by_slug(&slug).await.ok_or(AppError::NotFound)?;

    let content = state
        .s3_client
        .get_object(&format!("markdown/{}.md", slug))
        .await?;

    let (_, markdown_body) = parse_frontmatter(&content)?;
    let html_content = markdown_to_html(&markdown_body);

    let posts = state.cache.get_posts().await;
    let current_index = posts.iter().position(|p| p.slug == post.slug);
    let newer_post = current_index
        .and_then(|i| if i > 0 { posts.get(i - 1).cloned() } else { None });
    let older_post = current_index
        .and_then(|i| posts.get(i + 1).cloned());
    let post_number = current_index.map(|i| i + 1);
    let post_total = posts.len();

    let mut context = Context::new();
    context.insert("post", &post);
    context.insert("content", &html_content);
    context.insert("site_name", &state.config.site.name);
    context.insert("site_description", &state.config.site.description);
    context.insert("base_url", &state.config.site.base_url);
    context.insert("posts", &posts);
    context.insert("newer_post", &newer_post);
    context.insert("older_post", &older_post);
    context.insert("post_number", &post_number);
    context.insert("post_total", &post_total);
    context.insert("now_year", &chrono::Utc::now().format("%Y").to_string());

    let html = state.templates.render("post.html", &context)?;
    Ok(Html(html))
}

async fn refresh_cache_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let auth_header = headers.get("authorization").and_then(|h| h.to_str().ok());

    if !is_valid_token(auth_header, &state.config) {
        return Err(AppError::Unauthorized);
    }

    state.cache.refresh(&state.s3_client, &state.config).await?;
    info!("Cache refreshed via API");

    Ok((StatusCode::OK, "Cache refreshed"))
}

async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

async fn asset_proxy_handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<Response, AppError> {
    let key = format!("assets/{}", path);
    
    let bytes = state.s3_client.get_object_bytes(&key).await
        .map_err(|_| AppError::NotFound)?;

    let content_type = from_path(&path).first_or_octet_stream();
    let mut response = Response::new(bytes.into());
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(content_type.as_ref()).unwrap(),
    );
    
    // Cache for 1 year (immutable assets)
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    );

    Ok(response)
}

fn is_valid_token(auth_header: Option<&str>, config: &Config) -> bool {
    if config.api.refresh_token.is_empty() {
        return false;
    }
    auth_header
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(|t| t == config.api.refresh_token)
        .unwrap_or(false)
}

fn parse_frontmatter(content: &str) -> Result<(serde_json::Value, String), AppError> {
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return Err(AppError::InvalidFrontmatter);
    }
    let frontmatter_str = parts[1].trim();
    let body = parts[2].trim().to_string();
    let frontmatter: serde_json::Value = serde_yaml::from_str(frontmatter_str)?;
    Ok((frontmatter, body))
}

fn markdown_to_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);

    let parser = MdParser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received, starting graceful shutdown");
}