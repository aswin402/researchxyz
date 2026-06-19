use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryEntry {
    pub id: String,
    pub timestamp: String,
    pub query: String,
    pub summary: String,
    pub keywords: Vec<String>,
    pub sources: Vec<String>,
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
        let timestamp = chrono::Local::now().to_rfc3339();
        let id = format!("{}_{}", chrono::Utc::now().timestamp_millis(), keywords.first().cloned().unwrap_or_else(|| "entry".to_string()));
        
        let entry = MemoryEntry {
            id,
            timestamp,
            query: query.to_string(),
            summary: summary.to_string(),
            keywords,
            sources,
        };
        
        self.entries.push(entry);
        self.save()?;
        Ok(())
    }

    pub fn search(&self, query: &str, max_results: usize) -> Vec<MemoryEntry> {
        let query_words: Vec<String> = query.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| s.len() > 2)
            .map(|s| s.to_string())
            .collect();
            
        let mut scored_entries: Vec<(usize, MemoryEntry)> = self.entries.iter()
            .map(|entry| {
                let mut score = 0;
                // Check query words overlap with memory query
                let entry_query_lower = entry.query.to_lowercase();
                for word in &query_words {
                    if entry_query_lower.contains(word) {
                        score += 3; // query title match gets higher weight
                    }
                }
                
                // Check overlap with keywords
                for kw in &entry.keywords {
                    let kw_lower = kw.to_lowercase();
                    for word in &query_words {
                        if kw_lower.contains(word) {
                            score += 2;
                        }
                    }
                }
                
                // Check overlap with summary content
                let summary_lower = entry.summary.to_lowercase();
                for word in &query_words {
                    if summary_lower.contains(word) {
                        score += 1;
                    }
                }
                
                (score, entry.clone())
            })
            .filter(|(score, _)| *score > 0)
            .collect();
            
        // Sort by score descending
        scored_entries.sort_by(|a, b| b.0.cmp(&a.0));
        
        scored_entries.into_iter()
            .take(max_results)
            .map(|(_, entry)| entry)
            .collect()
    }
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

        // Reload to verify persistence
        let reloaded = MemoryManager::new_with_path(file_path.clone());
        assert_eq!(reloaded.entries.len(), 2);
        
        // Search overlap
        let results = reloaded.search("Rust borrow checker", 5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].query, "Rust memory safety guarantees");
        
        let results_python = reloaded.search("dynamic typing", 5);
        assert_eq!(results_python.len(), 1);
        assert_eq!(results_python[0].query, "Python dynamic typing");
        
        // Clean up
        let _ = std::fs::remove_file(&file_path);
    }
}
