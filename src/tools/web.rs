use crate::core::registry::Tool;
use crate::core::types::{ToolResult, ToolError, SourceRef};
use reqwest::Client;
use scraper::{Html, Selector};
use scraper::node::Node;
use serde_json::json;
use regex::Regex;
use std::time::Duration;

pub struct WebSearchTool {
    client: Client,
}

impl WebSearchTool {
    pub fn new() -> Self {
        WebSearchTool {
            client: Client::builder()
                .use_rustls_tls()
                .timeout(Duration::from_secs(15))
                .build()
                .unwrap_or_default(),
        }
    }

    async fn perform_search(&self, query: &str) -> Result<serde_json::Value, ToolError> {
        // 0. Try Websurfx Local/Private Search Engine API (if WEBSURFX_URL is set)
        if let Ok(websurfx_url) = std::env::var("WEBSURFX_URL") {
            if !websurfx_url.trim().is_empty() {
                let base = websurfx_url.trim().trim_end_matches('/');
                let encoded_query = percent_encoding::utf8_percent_encode(query, percent_encoding::NON_ALPHANUMERIC).to_string();
                let url = format!("{}/?q={}&json=true", base, encoded_query);
                
                if let Ok(res) = self.client.get(&url).send().await {
                    if res.status().is_success() {
                        if let Ok(resp_json) = res.json::<serde_json::Value>().await {
                            if let Some(results) = resp_json.get("results").and_then(|r| r.as_array()) {
                                let mut search_results = Vec::new();
                                for r in results {
                                    let title = r.get("title").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                                    let url = r.get("url").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                                    let snippet = r.get("content").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                                    search_results.push(json!({
                                        "title": title,
                                        "url": url,
                                        "snippet": snippet
                                    }));
                                }
                                if !search_results.is_empty() {
                                    return Ok(serde_json::Value::Array(search_results));
                                }
                            }
                        }
                    }
                }
            }
        }

        // 1. Try Tavily Search API (if TAVILY_API_KEY is set)
        if let Ok(tavily_key) = std::env::var("TAVILY_API_KEY") {
            if !tavily_key.trim().is_empty() {
                let body = json!({
                    "api_key": tavily_key,
                    "query": query,
                    "search_depth": "basic",
                    "max_results": 5
                });
                if let Ok(res) = self.client.post("https://api.tavily.com/search").json(&body).send().await {
                    if res.status().is_success() {
                        if let Ok(resp_json) = res.json::<serde_json::Value>().await {
                            if let Some(results) = resp_json.get("results").and_then(|r| r.as_array()) {
                                let mut search_results = Vec::new();
                                for r in results {
                                    let title = r.get("title").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                                    let url = r.get("url").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                                    let snippet = r.get("content").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                                    search_results.push(json!({
                                        "title": title,
                                        "url": url,
                                        "snippet": snippet
                                    }));
                                }
                                return Ok(serde_json::Value::Array(search_results));
                            }
                        }
                    }
                }
            }
        }

        // 2. Try Exa Search API (if EXA_API_KEY is set)
        if let Ok(exa_key) = std::env::var("EXA_API_KEY") {
            if !exa_key.trim().is_empty() {
                let body = json!({
                    "query": query,
                    "numResults": 5,
                    "useAutoprompt": true
                });
                if let Ok(res) = self.client.post("https://api.exa.ai/search").header("x-api-key", exa_key).json(&body).send().await {
                    if res.status().is_success() {
                        if let Ok(resp_json) = res.json::<serde_json::Value>().await {
                            if let Some(results) = resp_json.get("results").and_then(|r| r.as_array()) {
                                let mut search_results = Vec::new();
                                for r in results {
                                    let title = r.get("title").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                                    let url = r.get("url").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                                    let snippet = r.get("text").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                                    search_results.push(json!({
                                        "title": title,
                                        "url": url,
                                        "snippet": snippet
                                    }));
                                }
                                return Ok(serde_json::Value::Array(search_results));
                            }
                        }
                    }
                }
            }
        }

        // 3. Fallback to DuckDuckGo scraping
        let mut search_results = Vec::new();
        let mut ddg_success = false;

        let res = self.client.get("https://html.duckduckgo.com/html/")
            .query(&[("q", query)])
            .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .send()
            .await;

        if let Ok(response) = res {
            if response.status().is_success() {
                if let Ok(html_content) = response.text().await {
                    let document = Html::parse_document(&html_content);

                    if let (Ok(result_selector), Ok(title_selector), Ok(snippet_selector)) = (
                        Selector::parse(".result"),
                        Selector::parse(".result__title .result__a"),
                        Selector::parse(".result__snippet")
                    ) {
                        for element in document.select(&result_selector) {
                            let title = element.select(&title_selector)
                                .next()
                                .map(|e| e.text().collect::<String>().trim().to_string())
                                .unwrap_or_default();

                            let href = element.select(&title_selector)
                                .next()
                                .and_then(|e| e.value().attr("href"))
                                .map(|s| s.to_string())
                                .unwrap_or_default();

                            let snippet = element.select(&snippet_selector)
                                .next()
                                .map(|e| e.text().collect::<String>().trim().to_string())
                                .unwrap_or_default();

                            if !title.is_empty() && !href.is_empty() {
                                let clean_url = if href.contains("uddg=") {
                                    if let Some(pos) = href.find("uddg=") {
                                        let raw_url = &href[pos + 5..];
                                        percent_encoding::percent_decode_str(raw_url)
                                            .decode_utf8_lossy()
                                            .into_owned()
                                    } else {
                                        href
                                    }
                                } else if href.starts_with("//") {
                                    format!("https:{}", href)
                                } else {
                                    href
                                };

                                let clean_url = if let Some(pos) = clean_url.find("&rut=") {
                                    clean_url[..pos].to_string()
                                } else {
                                    clean_url
                                };

                                search_results.push(json!({
                                    "title": title,
                                    "url": clean_url,
                                    "snippet": snippet
                                }));
                            }
                        }
                        if !search_results.is_empty() {
                            ddg_success = true;
                        }
                    }
                }
            }
        }

        // 4. Try Mojeek scraping if DuckDuckGo fails or returns no results
        if !ddg_success {
            let res = self.client.get("https://www.mojeek.com/search")
                .query(&[("q", query)])
                .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                .send()
                .await;

            if let Ok(response) = res {
                if response.status().is_success() {
                    if let Ok(html_content) = response.text().await {
                        let document = Html::parse_document(&html_content);
                        if let (Ok(li_selector), Ok(title_selector), Ok(snippet_selector)) = (
                            Selector::parse("li"),
                            Selector::parse("a.title"),
                            Selector::parse("p.s")
                        ) {
                            for element in document.select(&li_selector) {
                                let title_node = element.select(&title_selector).next();
                                let snippet_node = element.select(&snippet_selector).next();

                                if let Some(tn) = title_node {
                                    let title = tn.text().collect::<String>().trim().to_string();
                                    let href = tn.value().attr("href").map(|s| s.to_string()).unwrap_or_default();
                                    let snippet = snippet_node
                                        .map(|e| e.text().collect::<String>().trim().to_string())
                                        .unwrap_or_default();

                                    if !title.is_empty() && !href.is_empty() {
                                        search_results.push(json!({
                                            "title": title,
                                            "url": href,
                                            "snippet": snippet
                                        }));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if search_results.is_empty() {
            return Err(ToolError::Upstream("All web search backends (Tavily, Exa, DuckDuckGo, Mojeek) failed or returned no results.".to_string()));
        }

        Ok(serde_json::Value::Array(search_results))
    }
}

#[async_trait::async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Perform a web search query and return a list of matching page titles, URLs, and snippets."
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "The search query term." }
            },
            "required": ["query"]
        })
    }

    async fn call(&self, input: serde_json::Value) -> Result<ToolResult, ToolError> {
        let query = input["query"].as_str().ok_or_else(|| {
            ToolError::InvalidInput("Missing 'query' parameter".to_string())
        })?;

        let search_res = self.perform_search(query).await?;
        
        let mut content = String::new();
        let mut citations = Vec::new();
        
        if let Some(arr) = search_res.as_array() {
            for (idx, r) in arr.iter().enumerate() {
                let title = r["title"].as_str().unwrap_or_default();
                let url = r["url"].as_str().unwrap_or_default();
                let snippet = r["snippet"].as_str().unwrap_or_default();
                
                let citation_id = (idx + 1) as u32;
                content.push_str(&format!(
                    "[{}] Title: {}\nURL: {}\nSnippet: {}\n\n",
                    citation_id, title, url, snippet
                ));
                
                citations.push(SourceRef {
                    id: citation_id,
                    url: Some(url.to_string()),
                    doi: None,
                    title: title.to_string(),
                });
            }
        }

        Ok(ToolResult { content, citations })
    }
}

pub struct WebFetchTool {
    client: Client,
}

impl WebFetchTool {
    pub fn new() -> Self {
        WebFetchTool {
            client: Client::builder()
                .use_rustls_tls()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }
}

fn walk_nodes(node: ego_tree::NodeRef<'_, Node>, text: &mut String) {
    match node.value() {
        Node::Text(t) => {
            text.push_str(&t.text);
        }
        Node::Element(e) => {
            let tag_name = e.name();
            if tag_name == "script" || tag_name == "style" || tag_name == "head" {
                return;
            }

            let is_block = matches!(
                tag_name,
                "p" | "div" | "br" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "li" | "tr" | "thead" | "tbody"
            );
            if is_block {
                text.push('\n');
            }
            for child in node.children() {
                walk_nodes(child, text);
            }
            if is_block {
                text.push('\n');
            }
        }
        _ => {
            for child in node.children() {
                walk_nodes(child, text);
            }
        }
    }
}

#[async_trait::async_trait]
impl Tool for WebFetchTool {
    fn name(&self) -> &str {
        "web_fetch"
    }

    fn description(&self) -> &str {
        "Fetch contents of a web page and return it as clean plain text."
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "The URL to fetch" }
            },
            "required": ["url"]
        })
    }

    async fn call(&self, input: serde_json::Value) -> Result<ToolResult, ToolError> {
        let url_str = input["url"].as_str().ok_or_else(|| {
            ToolError::InvalidInput("Missing 'url' argument".to_string())
        })?;

        let res = self.client.get(url_str)
            .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .send()
            .await
            .map_err(|e| ToolError::Network(e.to_string()))?;

        if !res.status().is_success() {
            return Err(ToolError::Upstream(format!("Failed to fetch URL: HTTP {}", res.status())));
        }

        let html = res.text().await.map_err(|e| ToolError::Upstream(e.to_string()))?;

        let result_text = {
            let document = Html::parse_document(&html);
            let mut raw_text = String::new();
            walk_nodes(document.tree.root(), &mut raw_text);

            let clean_text = raw_text
                .replace("&amp;", "&")
                .replace("&nbsp;", " ")
                .replace("&lt;", "<")
                .replace("&gt;", ">")
                .replace("&quot;", "\"")
                .replace("&#39;", "'");

            let re_whitespace = Regex::new(r" +").map_err(|e| ToolError::Upstream(e.to_string()))?;
            let re_newlines = Regex::new(r"\n\s*\n").map_err(|e| ToolError::Upstream(e.to_string()))?;
            let clean_text_spaces = re_whitespace.replace_all(&clean_text, " ");
            let final_text = re_newlines.replace_all(&clean_text_spaces, "\n");
            final_text.trim().to_string()
        };

        Ok(ToolResult {
            content: result_text,
            citations: vec![SourceRef {
                id: 1,
                url: Some(url_str.to_string()),
                doi: None,
                title: "Fetched Page".to_string(),
            }],
        })
    }
}
