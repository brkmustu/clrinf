use std::fs;
use std::path::{Path, PathBuf};

pub struct AiRuleGenerator;

pub struct AiRuleResult {
    pub rule_file: PathBuf,
    pub test_file: PathBuf,
}

impl AiRuleGenerator {
    pub fn generate_rule(
        project_path: &Path,
        _module: &str,
        rule_name: &str,
        command_name: &str,
        natural_description: &str,
    ) -> std::io::Result<AiRuleResult> {
        let rules_dir = project_path.join("src").join("rules");
        let target_rules_dir = if rules_dir.exists() {
            rules_dir
        } else {
            let crates_dir = project_path.join("crates");
            let mut found = project_path.join("src").join("rules");
            if crates_dir.exists() {
                if let Ok(entries) = fs::read_dir(crates_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
                        if dir_name.ends_with("-application") || dir_name.ends_with("-core") {
                            found = path.join("src").join("rules");
                            break;
                        }
                    }
                }
            }
            found
        };

        fs::create_dir_all(&target_rules_dir)?;

        let snake_name = to_snake_case(rule_name);
        let rule_file = target_rules_dir.join(format!("{snake_name}.rs"));

        let rule_code = format!(
            r#"use async_trait::async_trait;
use clrinf_core::{{BusinessRule, RequestContext, RuleResult}};

/// İş Kuralı: {natural_description}
pub struct {rule_name};

#[async_trait]
impl BusinessRule<{command_name}> for {rule_name} {{
    fn priority(&self) -> i32 {{
        1
    }}

    async fn evaluate(
        &self,
        command: &{command_name},
        request_context: &RequestContext,
    ) -> RuleResult {{
        // TODO: Kural koşulunu uygulayın
        // Örnek:
        // if command.price <= 0.0 {{
        //     return RuleResult::failed("INVALID_PRICE", "{natural_description}");
        // }}

        RuleResult::success()
    }}
}}
"#
        );
        fs::write(&rule_file, rule_code)?;

        // Test file
        let tests_dir = project_path.join("tests");
        fs::create_dir_all(&tests_dir)?;
        let test_file = tests_dir.join(format!("{snake_name}_test.rs"));

        let test_code = format!(
            r#"use clrinf_core::{{BusinessRule, RequestContext}};

#[tokio::test]
async fn test_{snake_name}_success() {{
    let context = RequestContext::new("tenant-default", "corr-1", "cause-1").unwrap();
    // let rule = {rule_name};
    // let command = {command_name} {{ ... }};
    // let result = rule.evaluate(&command, &context).await;
    // assert!(result.is_success);
}}
"#
        );
        fs::write(&test_file, test_code)?;

        Ok(AiRuleResult {
            rule_file,
            test_file,
        })
    }
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}
