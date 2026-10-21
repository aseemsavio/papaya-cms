use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use glob::glob;
use indicatif::{ProgressBar, ProgressStyle};
use tempfile::tempdir;
use tera::Tera;

use crate::error::AppError;

#[derive(Clone)]
pub struct TemplateEngine {
    tera: Arc<Tera>,
    template_dir: String,
}

impl TemplateEngine {
    pub async fn new(
        template_dir: &str,
        template_repo: &str,
        template_name: &str,
    ) -> Result<Self, AppError> {
        let template_path = Path::new(template_dir).join(template_name);

        if !template_path.exists() {
            Self::clone_template_repo(template_repo, &template_path).await?;
        }

        // Collect every template file first and register them in a single batch.
        // Registering one by one would break `{% extends %}` / `{% include %}`
        // whenever a child template is loaded before its parent/partial.
        // Canonicalising first keeps relative paths ("./templates/x") working.
        let root = std::fs::canonicalize(&template_path)?;
        let pattern = format!("{}/**/*", root.display());
        let mut sources: Vec<(String, String)> = Vec::new();
        for entry in glob(&pattern).map_err(|e| anyhow::anyhow!("glob pattern error: {}", e))? {
            let path = entry.map_err(|e| anyhow::anyhow!("glob entry error: {}", e))?;
            if path.is_file() {
                let content = std::fs::read_to_string(&path)?;
                let resolved = std::fs::canonicalize(&path).unwrap_or(path);
                let relative_path = resolved.strip_prefix(&root)
                    .map_err(|e| anyhow::anyhow!("strip prefix error: {}", e))?;
                let name = relative_path.to_string_lossy().replace('\\', "/");
                sources.push((name, content));
            }
        }

        if sources.is_empty() {
            return Err(AppError::Template(format!(
                "No template files found in {}",
                template_path.display()
            )));
        }

        let mut tera = Tera::default();
        tera.add_raw_templates(sources)?;

        Ok(Self {
            tera: Arc::new(tera),
            template_dir: template_path.to_string_lossy().to_string(),
        })
    }

    async fn clone_template_repo(repo_url: &str, target_path: &Path) -> Result<(), AppError> {
        let temp_dir = tempdir()?;
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message(format!("Cloning template repository from {}", repo_url));
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.transfer_progress(|stats| {
            pb.set_message(format!(
                "Cloning... {}/{} objects ({} KB)",
                stats.indexed_objects(),
                stats.total_objects(),
                stats.received_bytes() / 1024
            ));
            true
        });

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fetch_options);

        builder.clone(repo_url, temp_dir.path())
            .map_err(|e| AppError::Template(e.to_string()))?;

        let src_template_dir = temp_dir.path().join("templates").join(target_path.file_name().unwrap());

        if !src_template_dir.exists() {
            return Err(AppError::Template(format!(
                "Template '{}' not found in repository",
                target_path.file_name().unwrap().to_string_lossy()
            )));
        }

        std::fs::create_dir_all(target_path.parent().unwrap())?;
        copy_dir(&src_template_dir, target_path)?;

        pb.finish_with_message("Template cloned successfully");
        Ok(())
    }

    pub fn render(&self, name: &str, context: &tera::Context) -> Result<String, AppError> {
        self.tera.render(name, context).map_err(|e| AppError::Template(e.to_string()))
    }
}

fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}