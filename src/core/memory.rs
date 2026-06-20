use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    Fact,
    ToolFailure,
    LinkFailure,
    UserCorrection,
}

fn default_entry_type() -> EntryType {
    EntryType::Fact
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryEntry {
    pub id: String,
    pub timestamp: String,
    #[serde(default = "default_entry_type")]
    pub entry_type: EntryType,
    pub query: String,
    pub summary: String,
    pub keywords: Vec<String>,
    pub sources: Vec<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

pub struct MemoryManager {
    file_path: PathBuf,
    pub entries: Vec<MemoryEntry>,
}

impl MemoryManager {
    pub fn new_with_path(file_path: PathBuf) -> Self {
        let mut entries = Vec::new();
        if file_path.exists() {
            if let Ok(content) = fs::read_to_string(&file_path) {
                if let Ok(parsed) = serde_json::from_str::<Vec<MemoryEntry>>(&content) {
                    entries = parsed;
                }
            }
        }
        MemoryManager { file_path, entries }
    }

    pub fn load() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let file_path = PathBuf::from(home).join(".config/researchxyz/memory.json");
        Self::new_with_path(file_path)
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self.entries)?;
        fs::write(&self.file_path, content)?;
        Ok(())
    }

    pub fn add(&mut self, query: &str, summary: &str, keywords: Vec<String>, sources: Vec<String>) -> Result<(), anyhow::Error> {
        self.add_detailed(query, summary, keywords, sources, EntryType::Fact, serde_json::Value::Null)
    }

    pub fn add_detailed(
        &mut self,
        query: &str,
        summary: &str,
        keywords: Vec<String>,
        sources: Vec<String>,
        entry_type: EntryType,
        metadata: serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        // Try openmemory_rs if available
        if let Some(_) = get_openmemory_bin() {
            let status = match entry_type {
                EntryType::ToolFailure | EntryType::LinkFailure => "Failed",
                _ => "Success",
            };
            
            let reflection_args = serde_json::json!({
                "taskDescription": query,
                "status": status,
                "attemptNumber": 1,
                "stepsTaken": vec![format!("Log reflection for: {}", query)],
                "errorEncountered": if status == "Failed" { Some(summary) } else { None },
                "rootCause": if status == "Failed" { Some("API error or resource issue") } else { None },
                "solutionApplied": None::<String>,
                "reflection": summary
            });
            let _ = call_openmemory_tool_sync("log_reflection", reflection_args);

            let entity_type_str = match entry_type {
                EntryType::UserCorrection => "UserCorrection",
                EntryType::ToolFailure => "ToolFailure",
                EntryType::LinkFailure => "LinkFailure",
                EntryType::Fact => "Fact",
            };
            let entities_args = serde_json::json!({
                "entities": [
                    {
                        "name": query,
                        "entityType": entity_type_str,
                        "observations": vec![summary]
                    }
                ]
            });
            let _ = call_openmemory_tool_sync("create_entities", entities_args);
        }

        let timestamp = chrono::Local::now().to_rfc3339();
        let id = format!("{}_{}", chrono::Utc::now().timestamp_millis(), keywords.first().cloned().unwrap_or_else(|| "entry".to_string()));
        
        let entry = MemoryEntry {
            id,
            timestamp,
            entry_type,
            query: query.to_string(),
            summary: summary.to_string(),
            keywords,
            sources,
            metadata,
        };
        
        self.entries.push(entry);
        self.save()?;
        Ok(())
    }

    pub fn search(&self, query: &str, max_results: usize) -> Vec<MemoryEntry> {
        let mut results = Vec::new();
        
        // Try openmemory_rs if available
        if let Some(_) = get_openmemory_bin() {
            let mut openmemory_entries = Vec::new();

            // 1. Call retrieve_episodic_reflections
            let reflections_args = serde_json::json!({
                "query": query,
                "limit": max_results
            });
            if let Ok(res_val) = call_openmemory_tool_sync("retrieve_episodic_reflections", reflections_args) {
                if let Some(content_array) = res_val.get("content").and_then(|c| c.as_array()) {
                    if let Some(content_obj) = content_array.first() {
                        if let Some(text) = content_obj.get("text").and_then(|t| t.as_str()) {
                            if let Ok(reflections) = serde_json::from_str::<Vec<ReflectionItem>>(text) {
                                for item in reflections {
                                    let entry_type = if item.status == "Failed" {
                                        EntryType::ToolFailure
                                    } else if item.task_description.contains("/correct") || item.reflection.contains("/correct") {
                                        EntryType::UserCorrection
                                    } else {
                                        EntryType::Fact
                                    };
                                    
                                    let keywords = item.task_description.to_lowercase()
                                        .split(|c: char| !c.is_alphanumeric())
                                        .filter(|s| s.len() > 2)
                                        .map(|s| s.to_string())
                                        .collect();

                                    openmemory_entries.push(MemoryEntry {
                                        id: item.id,
                                        timestamp: item.created_at,
                                        entry_type,
                                        query: item.task_description,
                                        summary: item.reflection,
                                        keywords,
                                        sources: vec![],
                                        metadata: serde_json::json!({
                                            "steps_taken": item.steps_taken,
                                            "error_encountered": item.error_encountered,
                                            "root_cause": item.root_cause,
                                            "solution_applied": item.solution_applied
                                        }),
                                    });
                                }
                            }
                        }
                    }
                }
            }

            // 2. Call search_nodes
            let search_args = serde_json::json!({
                "query": query
            });
            if let Ok(res_val) = call_openmemory_tool_sync("search_nodes", search_args) {
                if let Some(content_array) = res_val.get("content").and_then(|c| c.as_array()) {
                    if let Some(content_obj) = content_array.first() {
                        if let Some(text) = content_obj.get("text").and_then(|t| t.as_str()) {
                            if let Ok(nodes) = serde_json::from_str::<Vec<OpenMemoryNode>>(text) {
                                for node in nodes {
                                    let entry_type = match node.entity_type.as_str() {
                                        "UserCorrection" => EntryType::UserCorrection,
                                        "ToolFailure" => EntryType::ToolFailure,
                                        "LinkFailure" => EntryType::LinkFailure,
                                        _ => EntryType::Fact,
                                    };

                                    openmemory_entries.push(MemoryEntry {
                                        id: format!("node_{}", node.name),
                                        timestamp: chrono::Local::now().to_rfc3339(),
                                        entry_type,
                                        query: node.name.clone(),
                                        summary: node.observations.join("\n"),
                                        keywords: vec![node.entity_type],
                                        sources: vec![],
                                        metadata: serde_json::Value::Null,
                                    });
                                }
                            }
                        }
                    }
                }
            }

            if !openmemory_entries.is_empty() {
                // Deduplicate by query/task description
                let mut unique_entries = Vec::new();
                for entry in openmemory_entries {
                    if !unique_entries.iter().any(|e: &MemoryEntry| e.query == entry.query) {
                        unique_entries.push(entry);
                    }
                }

                // Run overlap scoring to ensure correct relevance sorting and boosting
                let query_words: Vec<String> = query.to_lowercase()
                    .split(|c: char| !c.is_alphanumeric())
                    .filter(|s| s.len() > 2)
                    .map(|s| s.to_string())
                    .collect();

                let mut scored_entries: Vec<(usize, MemoryEntry)> = unique_entries.into_iter()
                    .map(|entry| {
                        let mut score = 0;
                        let entry_query_lower = entry.query.to_lowercase();
                        for word in &query_words {
                            if entry_query_lower.contains(word) {
                                score += 3;
                            }
                        }
                        for kw in &entry.keywords {
                            let kw_lower = kw.to_lowercase();
                            for word in &query_words {
                                if kw_lower.contains(word) {
                                    score += 2;
                                }
                            }
                        }
                        let summary_lower = entry.summary.to_lowercase();
                        for word in &query_words {
                            if summary_lower.contains(word) {
                                score += 1;
                            }
                        }
                        if score > 0 {
                            match entry.entry_type {
                                EntryType::UserCorrection => score += 5,
                                EntryType::ToolFailure | EntryType::LinkFailure => score += 2,
                                EntryType::Fact => {}
                            }
                        }
                        (score, entry)
                    })
                    .filter(|(score, _)| *score > 0)
                    .collect();

                scored_entries.sort_by(|a, b| b.0.cmp(&a.0));
                results = scored_entries.into_iter()
                    .take(max_results)
                    .map(|(_, entry)| entry)
                    .collect();
            }
        }

        // If openmemory was missing or returned no results, fall back to the JSON file
        if results.is_empty() {
            let query_words: Vec<String> = query.to_lowercase()
                .split(|c: char| !c.is_alphanumeric())
                .filter(|s| s.len() > 2)
                .map(|s| s.to_string())
                .collect();
                
            let mut scored_entries: Vec<(usize, MemoryEntry)> = self.entries.iter()
                .map(|entry| {
                    let mut score = 0;
                    let entry_query_lower = entry.query.to_lowercase();
                    for word in &query_words {
                        if entry_query_lower.contains(word) {
                            score += 3;
                        }
                    }
                    for kw in &entry.keywords {
                        let kw_lower = kw.to_lowercase();
                        for word in &query_words {
                            if kw_lower.contains(word) {
                                score += 2;
                            }
                        }
                    }
                    let summary_lower = entry.summary.to_lowercase();
                    for word in &query_words {
                        if summary_lower.contains(word) {
                            score += 1;
                        }
                    }
                    if score > 0 {
                        match entry.entry_type {
                            EntryType::UserCorrection => score += 5,
                            EntryType::ToolFailure | EntryType::LinkFailure => score += 2,
                            EntryType::Fact => {}
                        }
                    }
                    (score, entry.clone())
                })
                .filter(|(score, _)| *score > 0)
                .collect();
                
            scored_entries.sort_by(|a, b| b.0.cmp(&a.0));
            results = scored_entries.into_iter()
                .take(max_results)
                .map(|(_, entry)| entry)
                .collect();
        }

        results
    }
}

fn get_openmemory_bin() -> Option<PathBuf> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let bin_paths = [
        PathBuf::from(&home).join(".local/bin/openmemory_rs"),
        PathBuf::from("/home/aswin/programming/vscode/myProjects/ai_agent_tools/memory_rs/target/release/openmemory_rs"),
    ];
    bin_paths.iter().find(|p| p.exists()).cloned()
}

async fn call_openmemory_tool(tool_name: &str, arguments: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::process::Command;

    let bin_path = get_openmemory_bin()
        .ok_or_else(|| anyhow::anyhow!("openmemory_rs binary not found"))?;

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let db_dir = PathBuf::from(&home).join(".config/researchxyz");
    let db_path = db_dir.join("openmemory.db");
    if !db_dir.exists() {
        let _ = std::fs::create_dir_all(&db_dir);
    }

    let mut child = Command::new(bin_path)
        .env("MEMORY_DB_PATH", db_path.to_string_lossy().to_string())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    let mut stdin = child.stdin.take().ok_or_else(|| anyhow::anyhow!("Failed to open stdin"))?;
    let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("Failed to open stdout"))?;
    let mut reader = BufReader::new(stdout).lines();

    // 1. Initialize
    let init_req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "researchxyz",
                "version": "0.1.4"
            }
        },
        "id": 1
    });
    stdin.write_all(format!("{}\n", init_req).as_bytes()).await?;
    stdin.flush().await?;

    let mut init_res = String::new();
    if let Some(line) = reader.next_line().await? {
        init_res = line;
    }
    let _init_val: serde_json::Value = serde_json::from_str(&init_res)?;

    // 2. Initialized
    let initialized_notify = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    stdin.write_all(format!("{}\n", initialized_notify).as_bytes()).await?;
    stdin.flush().await?;

    // 3. Tool Call
    let call_req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": tool_name,
            "arguments": arguments
        },
        "id": 2
    });
    stdin.write_all(format!("{}\n", call_req).as_bytes()).await?;
    stdin.flush().await?;

    let mut call_res = String::new();
    if let Some(line) = reader.next_line().await? {
        call_res = line;
    }
    
    let _ = child.kill().await;

    let response_val: serde_json::Value = serde_json::from_str(&call_res)?;
    
    if let Some(error) = response_val.get("error") {
        if !error.is_null() {
            return Err(anyhow::anyhow!("openmemory_rs error: {}", error));
        }
    }

    let result = response_val.get("result")
        .ok_or_else(|| anyhow::anyhow!("Missing result field in JSON-RPC response"))?;
        
    Ok(result.clone())
}

fn call_openmemory_tool_sync(tool_name: &str, arguments: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.block_on(call_openmemory_tool(tool_name, arguments))
    } else {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        rt.block_on(call_openmemory_tool(tool_name, arguments))
    }
}

#[allow(dead_code)]
#[derive(serde::Deserialize, Debug, Clone)]
struct ReflectionItem {
    id: String,
    task_description: String,
    status: String,
    attempt_number: u32,
    steps_taken: Vec<String>,
    error_encountered: Option<String>,
    root_cause: Option<String>,
    solution_applied: Option<String>,
    reflection: String,
    created_at: String,
}

#[derive(serde::Deserialize, Debug, Clone)]
struct OpenMemoryNode {
    name: String,
    #[serde(rename = "entityType")]
    entity_type: String,
    observations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_add_and_search() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!("test_memory_{}.json", chrono::Utc::now().timestamp_millis()));
        
        // Clean up any old file
        if file_path.exists() {
            let _ = std::fs::remove_file(&file_path);
        }
        
        let mut manager = MemoryManager::new_with_path(file_path.clone());
        assert_eq!(manager.entries.len(), 0);
        
        // Add entries
        manager.add(
            "Rust memory safety guarantees",
            "Rust guarantees safety using borrow checker and lifetime rules.",
            vec!["rust".to_string(), "memory".to_string(), "safety".to_string()],
            vec!["https://rust-lang.org".to_string()]
        ).unwrap();
        
        manager.add(
            "Python dynamic typing",
            "Python uses dynamic typing and is garbage collected.",
            vec!["python".to_string(), "typing".to_string()],
            vec![]
        ).unwrap();

        // Add a User Correction entry
        manager.add_detailed(
            "ArXiv PDF format rules",
            "Always compile ArXiv reports using bullet points for key findings.",
            vec!["arxiv".to_string(), "pdf".to_string(), "format".to_string()],
            vec![],
            EntryType::UserCorrection,
            serde_json::json!({ "format": "bullets" })
        ).unwrap();

        // Reload to verify persistence
        let reloaded = MemoryManager::new_with_path(file_path.clone());
        assert_eq!(reloaded.entries.len(), 3);
        
        // Search overlap
        let results = reloaded.search("Rust borrow checker", 5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].query, "Rust memory safety guarantees");
        
        // Search for User Correction
        let results_correction = reloaded.search("ArXiv PDF format details", 5);
        assert_eq!(results_correction.len(), 1);
        assert_eq!(results_correction[0].entry_type, EntryType::UserCorrection);
        
        // Clean up
        let _ = std::fs::remove_file(&file_path);
    }
}
