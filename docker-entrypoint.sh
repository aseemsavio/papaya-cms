#!/bin/bash
set -e

# Papaya CMS Docker Entrypoint
# Handles template repository cloning and configuration

# Default values
DEFAULT_TEMPLATE_REPO="https://github.com/aseemsavio/papaya-cms"
DEFAULT_TEMPLATE_NAME="default"
DEFAULT_TEMPLATE_DIR="/templates"

# Parse arguments (passed as environment variables)
TEMPLATE_REPO="${PAPAYA_TEMPLATE_REPO:-$DEFAULT_TEMPLATE_REPO}"
TEMPLATE_NAME="${PAPAYA_TEMPLATE_NAME:-$DEFAULT_TEMPLATE_NAME}"
TEMPLATE_DIR="${PAPAYA_TEMPLATE_DIR:-$DEFAULT_TEMPLATE_DIR}"

echo "=========================================="
echo "Papaya CMS - Starting up"
echo "=========================================="
echo "Template Repo: $TEMPLATE_REPO"
echo "Template Name: $TEMPLATE_NAME"
echo "Template Dir: $TEMPLATE_DIR"
echo "=========================================="

# Function to clone template repository
clone_template() {
    local repo_url="$1"
    local target_dir="$2"
    local template_name="$3"

    echo "Cloning template repository..."
    
    # Create a temporary directory for cloning
    local temp_dir=$(mktemp -d)
    
    # Clone with depth=1 for faster clone
    if git clone --depth=1 --branch=main "$repo_url" "$temp_dir" 2>/dev/null || \
       git clone --depth=1 --branch=master "$repo_url" "$temp_dir" 2>/dev/null; then
        
        # Find the template directory
        local src_template_dir="$temp_dir/templates/$template_name"
        
        if [ -d "$src_template_dir" ]; then
            mkdir -p "$target_dir"
            # Copy template files
            cp -r "$src_template_dir"/* "$target_dir/"
            echo "Template '$template_name' cloned successfully to $target_dir"
        else
            echo "ERROR: Template '$template_name' not found in repository"
            echo "Available templates:"
            ls -la "$temp_dir/templates/" 2>/dev/null || echo "No templates directory found"
            rm -rf "$temp_dir"
            exit 1
        fi
        
        rm -rf "$temp_dir"
    else
        echo "ERROR: Failed to clone repository: $repo_url"
        rm -rf "$temp_dir"
        exit 1
    fi
}

# Clone template if directory doesn't exist or is empty
if [ ! -d "$TEMPLATE_DIR/$TEMPLATE_NAME" ] || [ -z "$(ls -A "$TEMPLATE_DIR/$TEMPLATE_NAME" 2>/dev/null)" ]; then
    clone_template "$TEMPLATE_REPO" "$TEMPLATE_DIR" "$TEMPLATE_NAME"
else
    echo "Template '$TEMPLATE_NAME' already exists at $TEMPLATE_DIR/$TEMPLATE_NAME"
fi

# Verify template files exist
if [ ! -f "$TEMPLATE_DIR/$TEMPLATE_NAME/home.html" ] || [ ! -f "$TEMPLATE_DIR/$TEMPLATE_NAME/post.html" ]; then
    echo "ERROR: Required template files (home.html, post.html) not found"
    exit 1
fi

echo "=========================================="
echo "Starting Papaya CMS server..."
echo "=========================================="

# Execute the main command
exec "$@"