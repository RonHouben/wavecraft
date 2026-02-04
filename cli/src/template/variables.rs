use anyhow::Result;
use heck::{ToPascalCase, ToSnakeCase, ToTitleCase};
use regex::Regex;

/// Template variables that get replaced during project generation.
#[derive(Debug, Clone)]
pub struct TemplateVariables {
    pub plugin_name: String,
    pub plugin_name_snake: String,
    pub plugin_name_pascal: String,
    pub plugin_name_title: String,
    pub vendor: String,
    pub email: Option<String>,
    pub url: Option<String>,
    pub sdk_version: String,
    pub year: String,
}

impl TemplateVariables {
    pub fn new(
        plugin_name: String,
        vendor: String,
        email: Option<String>,
        url: Option<String>,
        sdk_version: String,
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
            vendor,
            email,
            url,
            sdk_version,
            year,
        }
    }
    
    /// Applies template variables to a string, replacing all {{variable}} placeholders.
    pub fn apply(&self, content: &str) -> Result<String> {
        let mut result = content.to_string();
        
        // Replace all variables
        result = result.replace("{{plugin_name}}", &self.plugin_name);
        result = result.replace("{{plugin_name_snake}}", &self.plugin_name_snake);
        result = result.replace("{{plugin_name_pascal}}", &self.plugin_name_pascal);
        result = result.replace("{{plugin_name_title}}", &self.plugin_name_title);
        result = result.replace("{{vendor}}", &self.vendor);
        result = result.replace("{{sdk_version}}", &self.sdk_version);
        result = result.replace("{{year}}", &self.year);
        
        // Optional variables - replace with empty string if None
        result = result.replace("{{email}}", self.email.as_deref().unwrap_or(""));
        result = result.replace("{{url}}", self.url.as_deref().unwrap_or(""));
        
        // Check for unreplaced variables
        let unreplaced = Regex::new(r"\{\{(\w+)\}\}").unwrap();
        if let Some(captures) = unreplaced.captures(&result) {
            let var_name = &captures[1];
            anyhow::bail!("Unreplaced template variable: {{{{{}}}}}", var_name);
        }
        
        Ok(result)
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
            None,
            None,
            "0.7.0".to_string(),
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
            Some("info@example.com".to_string()),
            Some("https://example.com".to_string()),
            "0.7.0".to_string(),
        );
        
        let template = "# {{plugin_name_title}} by {{vendor}}";
        let result = vars.apply(template).unwrap();
        assert_eq!(result, "# My Plugin by My Company");
    }
    
    #[test]
    fn test_unreplaced_variable() {
        let vars = TemplateVariables::new(
            "my-plugin".to_string(),
            "My Company".to_string(),
            None,
            None,
            "0.7.0".to_string(),
        );
        
        let template = "Hello {{unknown_var}}";
        assert!(vars.apply(template).is_err());
    }
    
    #[test]
    fn test_empty_optional_variables() {
        let vars = TemplateVariables::new(
            "my-plugin".to_string(),
            "My Company".to_string(),
            None,  // No email
            None,  // No URL
            "0.7.0".to_string(),
        );
        
        // Template with optional variables should replace with empty string
        let template = "Email: {{email}}, URL: {{url}}";
        let result = vars.apply(template).unwrap();
        assert_eq!(result, "Email: , URL: ");
        
        // Should not error on templates with optional variables
        let template_with_url = "url: \"{{url}}\",";
        let result = vars.apply(template_with_url).unwrap();
        assert_eq!(result, "url: \"\",");
    }
}
