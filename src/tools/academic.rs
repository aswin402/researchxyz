use crate::core::registry::Tool;
use crate::core::types::{ToolResult, ToolError, SourceRef};
use serde_json::json;
use std::collections::HashSet;
use std::time::Duration;

pub struct AcademicSearchTool;

#[async_trait::async_trait]
impl Tool for AcademicSearchTool {
    fn name(&self) -> &str {
        "academic_search"
    }

    fn description(&self) -> &str {
        "Search academic databases (arXiv, Crossref, OpenAlex, Semantic Scholar) for literature research papers."
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "query": { "type": "string" },
                "sources": {
                    "type": "array",
                    "items": { "type": "string" },
                    "default": ["arxiv", "crossref", "openalex", "semantic_scholar"]
                },
                "max_results": { "type": "integer", "default": 10 }
            },
            "required": ["query"]
        })
    }

    async fn call(&self, input: serde_json::Value) -> Result<ToolResult, ToolError> {
        let query = input["query"].as_str().ok_or_else(|| {
            ToolError::InvalidInput("Missing query parameter".to_string())
        })?;
        
        let max_results = input["max_results"].as_u64().unwrap_or(10) as usize;
        let default_sources = vec![
            "arxiv".to_string(),
            "crossref".to_string(),
            "openalex".to_string(),
            "semantic_scholar".to_string(),
        ];
        
        let sources = input["sources"].as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<String>>())
            .unwrap_or(default_sources);

        // Load config to grab crossref mailto if available
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let config_path = std::path::PathBuf::from(home).join(".config/researchxyz/config.toml");
        let config = if config_path.exists() {
            crate::config::Config::load_from_path(&config_path).ok()
        } else {
            None
        };
        let mailto = config.as_ref().and_then(|c| c.academic.crossref_mailto.as_deref());

        let client = reqwest::Client::builder()
            .use_rustls_tls()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| ToolError::Network(e.to_string()))?;

        let mut all_results = Vec::new();

        // 1. arXiv
        if sources.contains(&"arxiv".to_string()) {
            let arxiv_results = search_arxiv(&client, query, max_results).await;
            all_results.extend(arxiv_results);
        }

        // 2. CrossRef
        if sources.contains(&"crossref".to_string()) {
            let crossref_results = search_crossref(&client, query, mailto, max_results).await;
            all_results.extend(crossref_results);
        }

        // 3. OpenAlex
        if sources.contains(&"openalex".to_string()) {
            let openalex_results = search_openalex(&client, query, max_results).await;
            all_results.extend(openalex_results);
        }

        // 4. Semantic Scholar
        if sources.contains(&"semantic_scholar".to_string()) {
            let sem_results = search_semantic_scholar(&client, query, max_results).await;
            all_results.extend(sem_results);
        }

        // Merge and deduplicate by DOI or title similarity
        let mut seen_titles = HashSet::new();
        let mut seen_dois = HashSet::new();
        let mut deduplicated = Vec::new();
        let mut id_counter = 1;

        for mut paper in all_results {
            let title_lower = paper.title.to_lowercase().trim().to_string();
            if title_lower.is_empty() || title_lower == "untitled" {
                continue;
            }
            let has_duplicate_title = seen_titles.contains(&title_lower);
            let has_duplicate_doi = paper.doi.as_ref().map(|d| seen_dois.contains(d)).unwrap_or(false);

            if !has_duplicate_title && !has_duplicate_doi {
                seen_titles.insert(title_lower);
                if let Some(ref d) = paper.doi {
                    seen_dois.insert(d.clone());
                }
                paper.id = id_counter;
                id_counter += 1;
                deduplicated.push(paper);
            }
        }

        if deduplicated.is_empty() {
            return Ok(ToolResult {
                content: format!("No academic papers found for query: {}", query),
                citations: vec![],
            });
        }

        let mut content = String::new();
        content.push_str("Academic research search results:\n\n");
        for paper in &deduplicated {
            content.push_str(&format!(
                "[{}] {}\n",
                paper.id, paper.title
            ));
            if let Some(ref url) = paper.url {
                content.push_str(&format!("   URL: {}\n", url));
            }
            if let Some(ref doi) = paper.doi {
                content.push_str(&format!("   DOI: {}\n", doi));
            }
            content.push_str("\n");
        }

        Ok(ToolResult {
            content,
            citations: deduplicated,
        })
    }
}

async fn search_arxiv(client: &reqwest::Client, query: &str, max_results: usize) -> Vec<SourceRef> {
    let encoded_query = percent_encoding::utf8_percent_encode(query, percent_encoding::NON_ALPHANUMERIC);
    let url = format!("http://export.arxiv.org/api/query?search_query=all:{}&max_results={}", encoded_query, max_results);
    let mut results = Vec::new();

    if let Ok(resp) = client.get(&url).header("User-Agent", concat!("ResearchXYZ/", env!("CARGO_PKG_VERSION"))).send().await {
        if let Ok(xml_text) = resp.text().await {
            let entry_re = regex::Regex::new(r"(?s)<entry>(.*?)</entry>").unwrap();
            let title_re = regex::Regex::new(r"(?s)<title>(.*?)</title>").unwrap();
            let id_re = regex::Regex::new(r"<id>(.*?)</id>").unwrap();
            let doi_re = regex::Regex::new(r#"<arxiv:doi[^>]*>(.*?)</arxiv:doi>"#).unwrap();

            for cap in entry_re.captures_iter(&xml_text) {
                let entry_content = &cap[1];
                let title = title_re.captures(entry_content)
                    .map(|c| c[1].trim().replace('\n', " "))
                    .unwrap_or_else(|| "Untitled".to_string());
                let raw_id = id_re.captures(entry_content)
                    .map(|c| c[1].trim().to_string())
                    .unwrap_or_default();
                let doi = doi_re.captures(entry_content)
                    .map(|c| c[1].trim().to_string());

                let url = if raw_id.is_empty() { None } else { Some(raw_id) };

                results.push(SourceRef {
                    id: 0,
                    url,
                    doi,
                    title,
                });
            }
        }
    }
    results
}

async fn search_crossref(client: &reqwest::Client, query: &str, mailto: Option<&str>, max_results: usize) -> Vec<SourceRef> {
    let encoded_query = percent_encoding::utf8_percent_encode(query, percent_encoding::NON_ALPHANUMERIC);
    let mut url = format!("https://api.crossref.org/works?query={}&rows={}", encoded_query, max_results);
    if let Some(email) = mailto {
        url.push_str(&format!("&mailto={}", email));
    }

    let mut results = Vec::new();
    if let Ok(resp) = client.get(&url).header("User-Agent", concat!("ResearchXYZ/", env!("CARGO_PKG_VERSION"))).send().await {
        if let Ok(json) = resp.json::<serde_json::Value>().await {
            if let Some(items) = json["message"]["items"].as_array() {
                for item in items {
                    let title = item["title"].as_array()
                        .and_then(|a| a.first())
                        .and_then(|v| v.as_str())
                        .unwrap_or("Untitled")
                        .to_string();
                    let doi = item["DOI"].as_str().map(|s| s.to_string());
                    let url = item["URL"].as_str().map(|s| s.to_string())
                        .or_else(|| doi.as_ref().map(|d| format!("https://doi.org/{}", d)));

                    results.push(SourceRef {
                        id: 0,
                        url,
                        doi,
                        title,
                    });
                }
            }
        }
    }
    results
}

async fn search_openalex(client: &reqwest::Client, query: &str, max_results: usize) -> Vec<SourceRef> {
    let encoded_query = percent_encoding::utf8_percent_encode(query, percent_encoding::NON_ALPHANUMERIC);
    let url = format!("https://api.openalex.org/works?search={}&per-page={}", encoded_query, max_results);
    let mut results = Vec::new();

    if let Ok(resp) = client.get(&url).header("User-Agent", concat!("ResearchXYZ/", env!("CARGO_PKG_VERSION"))).send().await {
        if let Ok(json) = resp.json::<serde_json::Value>().await {
            if let Some(results_array) = json["results"].as_array() {
                for item in results_array {
                    let title = item["title"].as_str().unwrap_or("Untitled").to_string();
                    let doi = item["doi"].as_str().map(|s| s.trim_start_matches("https://doi.org/").to_string());
                    let url = item["doi"].as_str().map(|s| s.to_string());

                    results.push(SourceRef {
                        id: 0,
                        url,
                        doi,
                        title,
                    });
                }
            }
        }
    }
    results
}

async fn search_semantic_scholar(client: &reqwest::Client, query: &str, max_results: usize) -> Vec<SourceRef> {
    let encoded_query = percent_encoding::utf8_percent_encode(query, percent_encoding::NON_ALPHANUMERIC);
    let url = format!("https://api.semanticscholar.org/graph/v1/paper/search?query={}&limit={}", encoded_query, max_results);
    let mut results = Vec::new();

    if let Ok(resp) = client.get(&url).header("User-Agent", concat!("ResearchXYZ/", env!("CARGO_PKG_VERSION"))).send().await {
        if let Ok(json) = resp.json::<serde_json::Value>().await {
            if let Some(data) = json["data"].as_array() {
                for item in data {
                    let title = item["title"].as_str().unwrap_or("Untitled").to_string();
                    let doi = item["externalIds"]["DOI"].as_str().map(|s| s.to_string());
                    let paper_id = item["paperId"].as_str();
                    let url = paper_id.map(|id| format!("https://www.semanticscholar.org/paper/{}", id));

                    results.push(SourceRef {
                        id: 0,
                        url,
                        doi,
                        title,
                    });
                }
            }
        }
    }
    results
}
