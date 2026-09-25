// @clrinf:generated — Template resolver abstraction for C# code generation
//
// Solves combinatorial explosion of template variations (paradigm, dispatcher, API style, pattern)
// by using a hierarchical physical folder layout instead of nested conditionals (no 'if is_fp', no 'if is_mediatr').
//
// Resolution Priority (Most Specific to General Fallback):
// 1. templates/csharp/{pattern}/{paradigm}/{dispatcher}/
// 2. templates/csharp/{pattern}/{paradigm}/
// 3. templates/csharp/{pattern}/
// 4. templates/csharp/

use crate::manifest::{ApiStyle, Dispatcher, Paradigm, PatternStyle};
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
use tera::Tera;

/// Configuration for resolving templates across architectural dimensions.
#[derive(Debug, Clone)]
pub struct TemplateResolutionContext<'a> {
    pub templates_dir: &'a Path,
    pub pattern: PatternStyle,
    pub paradigm: Paradigm,
    pub dispatcher: Dispatcher,
    pub api_style: ApiStyle,
}

/// Dynamic resolver that loads and cascades Tera templates without conditionals in template bodies.
pub struct TemplateResolver<'a> {
    ctx: TemplateResolutionContext<'a>,
}

impl<'a> TemplateResolver<'a> {
    pub fn new(ctx: TemplateResolutionContext<'a>) -> Self {
        Self { ctx }
    }

    /// Candidate directory paths in priority order (specific to general fallback).
    pub fn candidate_dirs(&self) -> Vec<PathBuf> {
        let pattern_str = self.ctx.pattern.as_str();
        let paradigm_str = self.ctx.paradigm.as_str();
        let dispatcher_str = self.ctx.dispatcher.as_str();
        let api_style_str = self.ctx.api_style.as_str();

        let base = self.ctx.templates_dir.join("csharp");
        let mut dirs = vec![
            base.join(pattern_str).join(paradigm_str).join(dispatcher_str).join(api_style_str),
            base.join(pattern_str).join(paradigm_str).join(dispatcher_str),
            base.join(pattern_str).join(paradigm_str),
            base.join(pattern_str),
        ];

        if self.ctx.pattern == PatternStyle::Flat {
            dirs.push(base.join("vertical-slice").join(paradigm_str).join(dispatcher_str).join(api_style_str));
            dirs.push(base.join("vertical-slice").join(paradigm_str).join(dispatcher_str));
            dirs.push(base.join("vertical-slice").join(paradigm_str));
            dirs.push(base.join("vertical-slice"));
        }

        dirs.push(base);
        dirs
    }

    /// Builds a Tera engine instance loaded with templates.
    /// Priority order: checks most specific directory first, falling back to general directories
    /// for any template not yet loaded. Non-recursive directory scan prevents flavor bleeding.
    pub fn build_tera(&self) -> Result<Tera> {
        let mut tera = Tera::default();
        let dirs = self.candidate_dirs();
        let mut loaded_any = false;

        for dir in &dirs {
            if dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("tera") {
                            if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                                if !tera.get_template_names().any(|name| name == file_name) {
                                    if let Ok(content) = std::fs::read_to_string(&path) {
                                        tera.add_raw_template(file_name, &content)?;
                                        loaded_any = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if !loaded_any {
            bail!(
                "No C# templates found in candidate paths for pattern='{}', paradigm='{}', dispatcher='{}' at: {:?}",
                self.ctx.pattern.as_str(),
                self.ctx.paradigm.as_str(),
                self.ctx.dispatcher.as_str(),
                dirs
            );
        }

        Ok(tera)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candidate_dirs_ordering() {
        let templates_dir = Path::new("/tmp/templates");
        let ctx = TemplateResolutionContext {
            templates_dir,
            pattern: PatternStyle::Flat,
            paradigm: Paradigm::Fp,
            dispatcher: Dispatcher::Native,
            api_style: ApiStyle::Minimal,
        };
        let resolver = TemplateResolver::new(ctx);
        let dirs = resolver.candidate_dirs();

        assert_eq!(dirs[0], PathBuf::from("/tmp/templates/csharp/flat/fp/native/minimal"));
        assert_eq!(dirs[1], PathBuf::from("/tmp/templates/csharp/flat/fp/native"));
        assert_eq!(dirs[2], PathBuf::from("/tmp/templates/csharp/flat/fp"));
        assert_eq!(dirs[3], PathBuf::from("/tmp/templates/csharp/flat"));
        assert_eq!(dirs[4], PathBuf::from("/tmp/templates/csharp/vertical-slice/fp/native/minimal"));
    }

    #[test]
    fn test_candidate_dirs_oop() {
        let templates_dir = Path::new("/tmp/templates");
        let ctx = TemplateResolutionContext {
            templates_dir,
            pattern: PatternStyle::Flat,
            paradigm: Paradigm::Oop,
            dispatcher: Dispatcher::Mediatr,
            api_style: ApiStyle::Controller,
        };
        let resolver = TemplateResolver::new(ctx);
        let dirs = resolver.candidate_dirs();

        assert_eq!(dirs[0], PathBuf::from("/tmp/templates/csharp/flat/oop/mediatr/controller"));
        assert_eq!(dirs[1], PathBuf::from("/tmp/templates/csharp/flat/oop/mediatr"));
        assert_eq!(dirs[2], PathBuf::from("/tmp/templates/csharp/flat/oop"));
    }
}
