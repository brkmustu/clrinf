use anyhow::{bail, ensure, Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Component, Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
pub struct TemplateManifest {
    pub name: String,
    pub description: String,
    pub language: String,
    pub kind: String,
    pub capabilities: Vec<String>,
    pub quickstart: Vec<String>,
}

pub struct TemplateManager {
    templates_root: PathBuf,
}

impl TemplateManager {
    pub fn new<P: AsRef<Path>>(templates_root: P) -> Self {
        Self {
            templates_root: templates_root.as_ref().to_path_buf(),
        }
    }

    fn root(&self) -> Result<PathBuf> {
        self.templates_root.canonicalize().with_context(|| format!(
            "Template catalog not found at {}. Pass --templates-dir /path/to/clrinf/templates (the installed CLI does not depend on its build checkout).",
            self.templates_root.display()
        ))
    }

    fn manifest(path: &Path) -> Result<TemplateManifest> {
        let file = path.join("clrinf-template.json");
        ensure!(
            !fs::symlink_metadata(&file)?.file_type().is_symlink(),
            "Template manifest cannot be a symlink: {}",
            file.display()
        );
        let manifest: TemplateManifest = serde_json::from_str(
            &fs::read_to_string(&file).with_context(|| format!("Read {}", file.display()))?,
        )
        .with_context(|| format!("Invalid template manifest {}", file.display()))?;
        ensure!(
            path.file_name().and_then(|n| n.to_str()) == Some(&manifest.name),
            "Manifest name must match directory: {}",
            path.display()
        );
        ensure!(
            !manifest.description.trim().is_empty()
                && !manifest.language.trim().is_empty()
                && !manifest.kind.trim().is_empty()
                && !manifest.quickstart.is_empty(),
            "Manifest requires description, language, kind and quickstart: {}",
            file.display()
        );
        Ok(manifest)
    }

    pub fn list_templates(&self) -> Result<Vec<(String, String, PathBuf)>> {
        let root = self.root()?;
        let mut list = Vec::new();
        for entry in fs::read_dir(&root)? {
            let entry = entry?;
            if entry.file_type()?.is_symlink() {
                bail!(
                    "Symlinked catalog entries are unsupported: {}",
                    entry.path().display()
                );
            }
            let path = entry.path();
            if path.is_dir() && path.join("clrinf-template.json").is_file() {
                let m = Self::manifest(&path)?;
                list.push((
                    m.name,
                    format!(
                        "{} [{}; {}; capabilities: {}]",
                        m.description,
                        m.language,
                        m.kind,
                        m.capabilities.join(", ")
                    ),
                    path,
                ));
            }
        }
        list.sort_by(|a, b| a.0.cmp(&b.0));
        ensure!(
            !list.is_empty(),
            "No clrinf-template.json manifests found in {}",
            root.display()
        );
        Ok(list)
    }

    pub fn scaffold(&self, template_name: &str, target_dir: &Path) -> Result<()> {
        let mut components = Path::new(template_name).components();
        ensure!(
            matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none(),
            "Template name must be a single directory name"
        );
        let root = self.root()?;
        let source = root.join(template_name);
        ensure!(
            !fs::symlink_metadata(&source)
                .with_context(|| format!(
                "Template '{template_name}' not found in {}. Run template list --templates-dir {}.",
                root.display(), root.display()
            ))?
                .file_type()
                .is_symlink(),
            "Template cannot be a symlink"
        );
        let source = source.canonicalize()?;
        ensure!(
            source.starts_with(&root),
            "Template must remain inside catalog"
        );
        let manifest = Self::manifest(&source)?;
        if target_dir.exists() {
            ensure!(
                !fs::symlink_metadata(target_dir)?.file_type().is_symlink(),
                "Target cannot be a symlink"
            );
            ensure!(
                target_dir.is_dir() && fs::read_dir(target_dir)?.next().is_none(),
                "Target must be an empty directory: {}",
                target_dir.display()
            );
        }
        let absolute_target = std::path::absolute(target_dir)?;
        let existing_parent = absolute_target
            .ancestors()
            .find(|path| path.exists())
            .context("Target has no existing parent")?
            .canonicalize()?;
        ensure!(
            !existing_parent.starts_with(&source),
            "Target must be outside source template"
        );
        // Inspect the complete source before writing; never follow template symlinks.
        let mut files = Vec::new();
        for entry in WalkDir::new(&source)
            .sort_by_file_name()
            .into_iter()
            .filter_entry(|e| {
                !matches!(
                    e.file_name().to_str(),
                    Some("node_modules" | "bin" | "obj" | "target" | ".git" | "_build" | "deps")
                )
            })
        {
            let entry = entry?;
            ensure!(
                !entry.file_type().is_symlink(),
                "Template symlink is unsupported: {}",
                entry.path().display()
            );
            if entry.file_type().is_file() {
                files.push(entry.path().strip_prefix(&source)?.to_path_buf());
            }
        }
        fs::create_dir_all(target_dir)?;
        let target = target_dir.canonicalize()?;
        ensure!(
            !target.starts_with(&source),
            "Target must be outside source template"
        );
        for relative in &files {
            let dest = target.join(relative);
            fs::create_dir_all(dest.parent().context("Missing destination parent")?)?;
            fs::copy(source.join(relative), dest)?;
        }
        println!(
            "Created {} from {} ({} files).",
            target_dir.display(),
            manifest.name,
            files.len()
        );
        println!("Next, in {}:", target_dir.display());
        for command in manifest.quickstart {
            println!("  {command}");
        }
        Ok(())
    }
}
