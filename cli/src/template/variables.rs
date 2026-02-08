use anyhow::Result;
use heck::{ToPascalCase, ToSnakeCase, ToTitleCase};
use regex::{escape, Regex};
use std::path::PathBuf;

/// Template variables that get replaced during project generation.
#[derive(Debug, Clone)]
pub struct TemplateVariables {
    pub plugin_name: String,
    pub plugin_name_snake: String,
    pub plugin_name_pascal: String,
    pub plugin_name_title: String,
    pub author_name: String,
    pub author_email: String,
    pub homepage: String,
    pub sdk_version: String,
    /// Local SDK path for development (generates path deps instead of git deps)
    pub local_dev: Option<PathBuf>,
    pub year: String,
}

impl TemplateVariables {
    pub fn new(
        plugin_name: String,
        author_name: String,
        author_email: String,
        homepage: String,
        sdk_version: String,
        local_dev: Option<PathBuf>,
    ) -> Self {
        let plugin_name_snake = plugin_name.to_snake_case();
        let plugin_name_pascal = plugin_name.to_pascal_case();
        let plugin_name_title = plugin_name.to_title_case();
        let year = chrono::Utc::now().format("%Y").to_string();

        Self {
            plugin_name,
            plugin_name_snake,
            plugin_name_pascal,
            plugin_name_title,
            author_name,
            author_email,
            homepage,
            sdk_version,
            local_dev,
            year,
        }
    }

    /// Applies template variables to a string, replacing all {{variable}} placeholders.
    pub fn apply(&self, content: &str) -> Result<String> {
        let mut result = content.to_string();

        // Replace all variables (allowing optional whitespace inside the braces)
        result = Self::replace_variable(result, "plugin_name", &self.plugin_name);
        result = Self::replace_variable(result, "plugin_name_snake", &self.plugin_name_snake);
        result = Self::replace_variable(result, "plugin_name_pascal", &self.plugin_name_pascal);
        result = Self::replace_variable(result, "plugin_name_title", &self.plugin_name_title);
        result = Self::replace_variable(result, "author_name", &self.author_name);
        result = Self::replace_variable(result, "author_email", &self.author_email);
        result = Self::replace_variable(result, "homepage", &self.homepage);
        result = Self::replace_variable(result, "sdk_version", &self.sdk_version);
        result = Self::replace_variable(result, "year", &self.year);

        // Check for unreplaced variables
        let unreplaced = Regex::new(r"\{\{\s*(\w+)\s*\}\}").unwrap();
        if let Some(captures) = unreplaced.captures(&result) {
            let var_name = &captures[1];
            anyhow::bail!("Unreplaced template variable: {{{{{}}}}}", var_name);
        }

        Ok(result)
    }

    fn replace_variable(result: String, name: &str, value: &str) -> String {
        let pattern = format!("\\{{\\{{\\s*{}\\s*\\}}\\}}", escape(name));
        let re = Regex::new(&pattern).expect("valid template variable regex");
        re.replace_all(&result, value).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_transformations() {
        let vars = TemplateVariables::new(
            "my-plugin".to_string(),
            "My Company".to_string(),
            "info@example.com".to_string(),
            "https://example.com".to_string(),
            "0.9.0".to_string(),
            None, // local_dev
        );

        assert_eq!(vars.plugin_name, "my-plugin");
        assert_eq!(vars.plugin_name_snake, "my_plugin");
        assert_eq!(vars.plugin_name_pascal, "MyPlugin");
        assert_eq!(vars.plugin_name_title, "My Plugin");
    }

    #[test]
    fn test_apply() {
        let vars = TemplateVariables::new(
            "my-plugin".to_string(),
            "My Company".to_string(),
            "info@example.com".to_string(),
            "https://example.com".to_string(),
            "0.9.0".to_string(),
            None, // local_dev
        );

        let template = "# {{plugin_name_title}} by {{author_name}}";
        let result = vars.apply(template).unwrap();
        assert_eq!(result, "# My Plugin by My Company");
    }

    #[test]
    fn test_unreplaced_variable() {
        let vars = TemplateVariables::new(
            "my-plugin".to_string(),
            "My Company".to_string(),
            "info@example.com".to_string(),
            "https://example.com".to_string(),
            "0.9.0".to_string(),
            None, // local_dev
        );

        let template = "Hello {{unknown_var}}";
        assert!(vars.apply(template).is_err());
    }

    #[test]
    fn test_metadata_fields() {
        let vars = TemplateVariables::new(
            "my-plugin".to_string(),
            "Developer Name".to_string(),
            "dev@example.com".to_string(),
            "https://myplugin.com".to_string(),
            "0.9.0".to_string(),
            None, // local_dev
        );

        // Test Cargo.toml author field template
        let template = r#"authors = ["{{author_name}} <{{author_email}}>"]"#;
        let result = vars.apply(template).unwrap();
        assert_eq!(result, r#"authors = ["Developer Name <dev@example.com>"]"#);

        // Test homepage field
        let template = r#"homepage = "{{homepage}}""#;
        let result = vars.apply(template).unwrap();
        assert_eq!(result, r#"homepage = "https://myplugin.com""#);
    }
}
