use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub llm: LlmConfig,
    pub output: OutputConfig,
    pub search: SearchConfig,
    pub academic: AcademicConfig,
    #[serde(default)]
    pub mcp: McpConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LlmConfig {
    pub provider: String,
    pub model: String,
    pub api_key_env: String,
    pub api_base: Option<String>,
    pub api_key: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OutputConfig {
    pub dir: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SearchConfig {
    pub backend: String,
    pub searxng_url: Option<String>,
    pub brave_api_key_env: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AcademicConfig {
    pub sources: Vec<String>,
    pub crossref_mailto: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct McpConfig {
    #[serde(default)]
    pub servers: Vec<McpServerConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct McpServerConfig {
    pub alias: String,
    pub command: String,
    pub args: Vec<String>,
}

impl Config {
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, anyhow::Error> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_path<P: AsRef<Path>>(&self, path: P) -> Result<(), anyhow::Error> {
        let content = toml::to_string_pretty(self)?;
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }

    pub fn default_config() -> Self {
        Self {
            llm: LlmConfig {
                provider: "anthropic".to_string(),
                model: "claude-sonnet-4-6".to_string(),
                api_key_env: "RESEARCHXYZ_API_KEY".to_string(),
                api_base: None,
                api_key: None,
            },
            output: OutputConfig {
                dir: "~/researchxyz-output".to_string(),
            },
            search: SearchConfig {
                backend: "duckduckgo".to_string(),
                searxng_url: None,
                brave_api_key_env: None,
            },
            academic: AcademicConfig {
                sources: vec![
                    "arxiv".to_string(),
                    "crossref".to_string(),
                    "openalex".to_string(),
                    "semantic_scholar".to_string(),
                ],
                crossref_mailto: None,
            },
            mcp: McpConfig { servers: vec![] },
        }
    }

    pub fn resolve_output_dir(&self) -> PathBuf {
        let path_str = &self.output.dir;
        if path_str.starts_with("~/") {
            if let Some(home_dir) = dirs_next::home_dir() {
                return home_dir.join(&path_str[2..]);
            }
        }
        PathBuf::from(path_str)
    }
}

pub fn run_configure_wizard() -> Result<(), anyhow::Error> {
    use inquire::{Select, Text, Password};

    println!("\n=============================================");
    println!("     ResearchXYZ Configuration Wizard        ");
    println!("=============================================\n");

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let config_path = PathBuf::from(&home).join(".config/researchxyz/config.toml");
    
    let mut config = if config_path.exists() {
        Config::load_from_path(&config_path).unwrap_or_else(|_| Config::default_config())
    } else {
        Config::default_config()
    };

    // 1. Select Provider
    let providers = vec![
        "anthropic",
        "openai",
        "deepseek",
        "groq",
        "openrouter",
        "google_ai_studio",
        "auto",
    ];
    let selected_provider = Select::new("Select LLM Provider:", providers).prompt()?;
    config.llm.provider = selected_provider.to_string();

    // 2. Determine default model
    let default_model = match selected_provider {
        "anthropic" => "claude-3-5-sonnet-latest",
        "openai" => "gpt-4o",
        "deepseek" => "deepseek-chat",
        "groq" => "llama-3.3-70b-versatile",
        "openrouter" => "google/gemini-2.0-flash-exp:free",
        "google_ai_studio" => "gemini-2.0-flash",
        _ => "gpt-4o",
    };

    // 3. Ask for model
    let model_input = Text::new(&format!("Enter model name (default: {}):", default_model))
        .with_default(default_model)
        .prompt()?;
    config.llm.model = model_input;

    // 4. Ask for direct API key (Masked/Password)
    let current_key_masked = if config.llm.api_key.is_some() { "[configured]" } else { "[not set]" };
    let api_key_input = Password::new(&format!("Paste API Key (current: {}):", current_key_masked))
        .without_confirmation()
        .prompt()?;
    
    if !api_key_input.trim().is_empty() {
        config.llm.api_key = Some(api_key_input.trim().to_string());
    }

    // 5. Ask for custom base URL (Optional)
    let default_base_prompt = config.llm.api_base.clone().unwrap_or_else(|| "[default for provider]".to_string());
    let api_base_input = Text::new(&format!("Enter API Base URL (press Enter to keep {}):", default_base_prompt))
        .prompt()?;
    
    if !api_base_input.trim().is_empty() {
        config.llm.api_base = Some(api_base_input.trim().to_string());
    }

    // Save the configuration
    config.save_to_path(&config_path)?;
    println!("\n✓ Configuration successfully saved to: {}\n", config_path.display());

    Ok(())
}

// Minimal dirs_next module to avoid adding dependency
mod dirs_next {
    use std::path::PathBuf;
    pub fn home_dir() -> Option<PathBuf> {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}
