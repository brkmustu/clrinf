use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Inject a `using` directive at the top of a C# file (after existing usings).
/// Skips if the using already exists.
pub fn add_using(file_path: &Path, using_line: &str) -> Result<bool> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read {}", file_path.display()))?;

    let trimmed = using_line.trim();
    if content.contains(trimmed) {
        return Ok(false);
    }

    // Find the last using line position
    let lines: Vec<&str> = content.lines().collect();
    let mut last_using_idx = None;
    for (i, line) in lines.iter().enumerate() {
        let l = line.trim();
        if l.starts_with("using ") && l.ends_with(';') {
            last_using_idx = Some(i);
        }
    }

    let mut new_lines: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
    match last_using_idx {
        Some(idx) => {
            new_lines.insert(idx + 1, trimmed.to_string());
        }
        None => {
            // No existing usings — insert at position 0 or after namespace
            new_lines.insert(0, trimmed.to_string());
        }
    }

    let new_content = new_lines.join("\n");
    fs::write(file_path, &new_content)
        .with_context(|| format!("Failed to write {}", file_path.display()))?;
    Ok(true)
}

/// Add a `DbSet<Entity>` property line to a DbContext file.
/// Inserts after the last existing `DbSet` property, or before the closing brace.
pub fn add_dbset_property(file_path: &Path, entity_name: &str, plural_name: &str) -> Result<bool> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read {}", file_path.display()))?;

    let dbset_line = format!("    public DbSet<{}> {} {{ get; set; }}", entity_name, plural_name);
    let dbset_check = format!("DbSet<{}>", entity_name);

    if content.contains(&dbset_check) {
        return Ok(false);
    }

    let lines: Vec<&str> = content.lines().collect();
    let mut last_dbset_idx = None;
    let mut last_close_brace_idx = None;

    for (i, line) in lines.iter().enumerate() {
        if line.contains("DbSet<") {
            last_dbset_idx = Some(i);
        }
        if line.trim() == "}" {
            last_close_brace_idx = Some(i);
        }
    }

    let insert_idx = last_dbset_idx
        .map(|i| i + 1)
        .or(last_close_brace_idx)
        .unwrap_or(lines.len());

    let mut new_lines: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
    new_lines.insert(insert_idx, dbset_line);

    let new_content = new_lines.join("\n");
    fs::write(file_path, &new_content)
        .with_context(|| format!("Failed to write {}", file_path.display()))?;
    Ok(true)
}

/// Add a line to a method body in a C# file.
/// Searches for the method by name, inserts the code before the method's closing brace.
#[allow(dead_code)]
pub fn add_line_to_method(file_path: &Path, method_name: &str, code_line: &str) -> Result<bool> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read {}", file_path.display()))?;

    let trimmed = code_line.trim();
    if content.contains(trimmed) {
        return Ok(false);
    }

    let lines: Vec<&str> = content.lines().collect();
    let mut method_found = false;
    let mut brace_depth: i32 = 0;
    let mut insert_idx = None;

    for (i, line) in lines.iter().enumerate() {
        if !method_found && line.contains(method_name) && (line.contains('(') || line.contains('{')) {
            method_found = true;
            if line.contains('{') {
                brace_depth = 1;
            }
            continue;
        }

        if method_found {
            for ch in line.chars() {
                if ch == '{' {
                    brace_depth += 1;
                } else if ch == '}' {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        insert_idx = Some(i);
                        break;
                    }
                }
            }
            if brace_depth == 0 && insert_idx.is_some() {
                break;
            }
            // Handle opening brace on next line
            if brace_depth == 0 && line.trim() == "{" {
                brace_depth = 1;
            }
        }
    }

    if let Some(idx) = insert_idx {
        let mut new_lines: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
        new_lines.insert(idx, format!("        {}", trimmed));
        let new_content = new_lines.join("\n");
        fs::write(file_path, &new_content)
            .with_context(|| format!("Failed to write {}", file_path.display()))?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Replace namespace prefixes in a generated file.
/// Handles the dynamic `project_namespace` injection for layer names.
#[allow(dead_code)]
pub fn apply_namespace_prefix(file_path: &Path, project_namespace: &str, folder_name: &str) -> Result<()> {
    if !file_path.exists() {
        return Ok(());
    }

    let mut content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read {}", file_path.display()))?;

    let layers = ["Application", "Domain", "Persistence", "Api", "Infrastructure"];

    // Adjust Features folder naming
    if folder_name != "Features" {
        if folder_name.is_empty() {
            content = content
                .replace("Application.Features.", "Application.")
                .replace("Application.Features", "Application");
        } else {
            content = content
                .replace("Application.Features.", &format!("Application.{}.", folder_name))
                .replace("Application.Features", &format!("Application.{}", folder_name));
        }
    }

    // Prefix layer namespaces with project namespace
    if !project_namespace.is_empty() && project_namespace != "Application" {
        for layer in &layers {
            content = content
                .replace(&format!("namespace {}", layer), &format!("namespace {}.{}", project_namespace, layer))
                .replace(&format!("using static {}", layer), &format!("using static {}.{}", project_namespace, layer))
                .replace(&format!("using {}", layer), &format!("using {}.{}", project_namespace, layer))
                .replace(&format!("= {}.", layer), &format!("= {}.{}.", project_namespace, layer));
        }
    }

    fs::write(file_path, &content)
        .with_context(|| format!("Failed to write {}", file_path.display()))?;
    Ok(())
}

/// Find the DbContext file in a project, searching common locations.
pub fn find_dbcontext(project_path: &Path, persistence_layer: &str, context_name: &str) -> Option<PathBuf> {
    let candidates = [
        project_path.join(persistence_layer).join("Contexts").join(format!("{}.cs", context_name)),
        project_path.join(persistence_layer).join("Data").join(format!("{}.cs", context_name)),
        project_path.join(persistence_layer).join(format!("{}.cs", context_name)),
        project_path.join("Data").join(format!("{}.cs", context_name)),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return Some(candidate.clone());
        }
    }

    // Fallback: recursive search
    for entry in walkdir::WalkDir::new(project_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_name().to_string_lossy() == format!("{}.cs", context_name) {
            let path_str = entry.path().to_string_lossy();
            if !path_str.contains("/bin/") && !path_str.contains("/obj/") {
                return Some(entry.path().to_path_buf());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_add_using() -> Result<()> {
        let dir = tempdir()?;
        let file = dir.path().join("Test.cs");
        fs::write(&file, "using System;\n\nnamespace App;\nclass C {}\n")?;

        let added = add_using(&file, "using Microsoft.EntityFrameworkCore;")?;
        assert!(added);

        let content = fs::read_to_string(&file)?;
        assert!(content.contains("using Microsoft.EntityFrameworkCore;"));

        // Idempotency check
        let second_run = add_using(&file, "using Microsoft.EntityFrameworkCore;")?;
        assert!(!second_run);
        Ok(())
    }

    #[test]
    fn test_add_dbset_property() -> Result<()> {
        let dir = tempdir()?;
        let file = dir.path().join("AppDbContext.cs");
        fs::write(&file, "public class AppDbContext : DbContext\n{\n    public DbSet<User> Users { get; set; }\n}\n")?;

        let added = add_dbset_property(&file, "Product", "Products")?;
        assert!(added);

        let content = fs::read_to_string(&file)?;
        assert!(content.contains("public DbSet<Product> Products { get; set; }"));

        // Duplicate check
        let duplicate = add_dbset_property(&file, "Product", "Products")?;
        assert!(!duplicate);
        Ok(())
    }
}

