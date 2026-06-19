use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub llm: LlmConfig,
    pub output: OutputConfig,
    pub search: SearchConfig,
    pub academic: AcademicConfig,
    #[serde(default)]
    pub mcp: McpConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LlmConfig {
    pub provider: String,
    pub model: String,
    pub api_key_env: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OutputConfig {
    pub dir: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SearchConfig {
    pub backend: String,
    pub searxng_url: Option<String>,
    pub brave_api_key_env: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AcademicConfig {
    pub sources: Vec<String>,
    pub crossref_mailto: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct McpConfig {
    #[serde(default)]
    pub servers: Vec<McpServerConfig>,
}

#[derive(Debug, Deserialize, Clone)]
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

    pub fn default_config() -> Self {
        Self {
            llm: LlmConfig {
                provider: "anthropic".to_string(),
                model: "claude-sonnet-4-6".to_string(),
                api_key_env: "RESEARCHXYZ_API_KEY".to_string(),
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

// Minimal dirs_next module to avoid adding dependency
mod dirs_next {
    use std::path::PathBuf;
    pub fn home_dir() -> Option<PathBuf> {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}
