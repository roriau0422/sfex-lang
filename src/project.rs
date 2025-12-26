// Copyright 2025 Temuujin
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Default)]
pub struct ProjectManifest {
    pub package: Option<PackageInfo>,
    pub dependencies: Option<HashMap<String, DependencySpec>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PackageInfo {
    pub name: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum DependencySpec {
    Path { path: String },
    Git { git: String },
    Simple(String),
}

pub fn find_project_root(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(dir) = current {
        if dir.join("sfex.toml").exists() {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }
    None
}

pub fn load_manifest(root: &Path) -> Result<ProjectManifest, String> {
    let manifest_path = root.join("sfex.toml");
    let contents = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read {}: {}", manifest_path.display(), e))?;
    toml::from_str(&contents).map_err(|e| format!("Failed to parse sfex.toml: {}", e))
}

pub fn packages_dir(root: &Path) -> PathBuf {
    root.join("packages")
}

pub fn resolve_module_path(module_path: &str, cwd: &Path) -> Option<PathBuf> {
    let raw_path = PathBuf::from(module_path);
    if raw_path.is_absolute() && raw_path.exists() {
        return Some(raw_path);
    }

    let direct = cwd.join(module_path);
    if direct.exists() {
        return Some(direct);
    }

    let root = find_project_root(cwd)?;
    let packaged = packages_dir(&root).join(module_path);
    if packaged.exists() {
        return Some(packaged);
    }

    None
}

pub fn install_dependencies(root: &Path) -> Result<Vec<String>, String> {
    let manifest = load_manifest(root)?;
    let dependencies = manifest.dependencies.unwrap_or_default();
    if dependencies.is_empty() {
        return Ok(Vec::new());
    }

    let packages_dir = packages_dir(root);
    std::fs::create_dir_all(&packages_dir)
        .map_err(|e| format!("Failed to create packages directory: {}", e))?;

    let mut installed = Vec::new();

    for (name, spec) in dependencies {
        let destination = packages_dir.join(&name);
        if destination.exists() {
            continue;
        }

        match spec {
            DependencySpec::Path { path } => {
                let source = root.join(path);
                copy_dir_recursive(&source, &destination)?;
                installed.push(name);
            }
            DependencySpec::Git { git } => {
                let status = std::process::Command::new("git")
                    .arg("clone")
                    .arg(&git)
                    .arg(&destination)
                    .status()
                    .map_err(|e| format!("Failed to run git: {}", e))?;

                if !status.success() {
                    return Err(format!("git clone failed for {}", git));
                }
                installed.push(name);
            }
            DependencySpec::Simple(_) => {
                return Err(format!(
                    "Dependency '{}' must specify a path or git URL",
                    name
                ));
            }
        }
    }

    Ok(installed)
}

fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<(), String> {
    if !source.exists() {
        return Err(format!(
            "Dependency path '{}' does not exist",
            source.display()
        ));
    }

    if source.is_file() {
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }
        std::fs::copy(source, destination).map_err(|e| format!("Failed to copy file: {}", e))?;
        return Ok(());
    }

    std::fs::create_dir_all(destination)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    for entry in std::fs::read_dir(source)
        .map_err(|e| format!("Failed to read directory '{}': {}", source.display(), e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        let dest_path = destination.join(entry.file_name());

        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            if let Some(parent) = dest_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }
            std::fs::copy(&path, &dest_path).map_err(|e| format!("Failed to copy file: {}", e))?;
        }
    }

    Ok(())
}
