use crate::core::registry::Tool;
use crate::core::types::{ToolResult, ToolError};
use serde_json::json;
use std::fs;
use std::path::PathBuf;

fn get_output_path(title: &str, provided_filename: Option<&str>, ext: &str) -> Result<PathBuf, ToolError> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let config_path = std::path::PathBuf::from(home).join(".config/researchxyz/config.toml");
    let config = crate::config::Config::load_from_path(&config_path)
        .unwrap_or_else(|_| crate::config::Config::default_config());
    let output_dir = config.resolve_output_dir();
    fs::create_dir_all(&output_dir).map_err(|e| ToolError::Upstream(format!("Failed to create output dir: {}", e)))?;

    let name = if let Some(f) = provided_filename {
        if f.trim().is_empty() {
            None
        } else {
            Some(f.trim().to_string())
        }
    } else {
        None
    };

    let final_name = name.unwrap_or_else(|| {
        let slug: String = title.chars()
            .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
            .collect();
        // Remove consecutive underscores
        let mut clean_slug = String::new();
        let mut last_was_under = false;
        for c in slug.chars() {
            if c == '_' {
                if !last_was_under {
                    clean_slug.push(c);
                    last_was_under = true;
                }
            } else {
                clean_slug.push(c);
                last_was_under = false;
            }
        }
        let clean_slug = clean_slug.trim_matches('_').to_string();
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        format!("{}_{}", clean_slug, timestamp)
    });

    let mut path = output_dir.join(final_name);
    if path.extension().is_none() {
        path.set_extension(ext);
    }
    Ok(path)
}

pub struct CreateDocxTool;

#[async_trait::async_trait]
impl Tool for CreateDocxTool {
    fn name(&self) -> &str {
        "create_docx"
    }

    fn description(&self) -> &str {
        "Generate a Microsoft Word document (.docx) of the research summary."
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "title": { "type": "string" },
                "sections": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "heading": { "type": "string" },
                            "body": { "type": "string" }
                        },
                        "required": ["heading", "body"]
                    }
                },
                "citations": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": { "type": "integer" },
                            "title": { "type": "string" },
                            "url_or_doi": { "type": "string" }
                        },
                        "required": ["id", "title", "url_or_doi"]
                    }
                },
                "filename": { "type": "string" }
            },
            "required": ["title", "sections", "citations"]
        })
    }

    async fn call(&self, input: serde_json::Value) -> Result<ToolResult, ToolError> {
        let title = input["title"].as_str().ok_or_else(|| {
            ToolError::InvalidInput("Missing title".to_string())
        })?;
        let sections = input["sections"].as_array().ok_or_else(|| {
            ToolError::InvalidInput("Missing sections array".to_string())
        })?;
        let citations = input["citations"].as_array().ok_or_else(|| {
            ToolError::InvalidInput("Missing citations array".to_string())
        })?;
        let filename = input["filename"].as_str();

        let output_path = get_output_path(title, filename, "docx")?;

        use docx_rs::{Docx, Paragraph, Run};
        let mut doc = Docx::new();

        // Title
        doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(title).size(36).bold()));
        doc = doc.add_paragraph(Paragraph::new()); // spacing

        // Sections
        for sec in sections {
            let heading = sec["heading"].as_str().unwrap_or("Untitled Section");
            let body = sec["body"].as_str().unwrap_or("");
            doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(heading).size(28).bold()));
            doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(body)));
            doc = doc.add_paragraph(Paragraph::new()); // spacing
        }

        // References
        if !citations.is_empty() {
            doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text("References").size(24).bold()));
            for cit in citations {
                let id = cit["id"].as_i64().unwrap_or(0);
                let t = cit["title"].as_str().unwrap_or("Unknown Source");
                let link = cit["url_or_doi"].as_str().unwrap_or("");
                doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(format!("[{}] {} — {}", id, t, link))));
            }
        }

        let file = std::fs::File::create(&output_path)
            .map_err(|e| ToolError::Upstream(format!("Could not create docx file: {}", e)))?;
        doc.build().pack(file)
            .map_err(|e| ToolError::Upstream(format!("Could not generate docx contents: {}", e)))?;

        Ok(ToolResult {
            content: format!("Word document successfully generated at: {}", output_path.display()),
            citations: vec![],
        })
    }
}

pub struct CreatePdfTool;

#[async_trait::async_trait]
impl Tool for CreatePdfTool {
    fn name(&self) -> &str {
        "create_pdf"
    }

    fn description(&self) -> &str {
        "Generate a formatted PDF report of the research summary."
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "title": { "type": "string" },
                "sections": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "heading": { "type": "string" },
                            "body": { "type": "string" }
                        },
                        "required": ["heading", "body"]
                    }
                },
                "citations": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": { "type": "integer" },
                            "title": { "type": "string" },
                            "url_or_doi": { "type": "string" }
                        },
                        "required": ["id", "title", "url_or_doi"]
                    }
                },
                "filename": { "type": "string" }
            },
            "required": ["title", "sections", "citations"]
        })
    }

    async fn call(&self, input: serde_json::Value) -> Result<ToolResult, ToolError> {
        let title = input["title"].as_str().ok_or_else(|| {
            ToolError::InvalidInput("Missing title".to_string())
        })?;
        let sections = input["sections"].as_array().ok_or_else(|| {
            ToolError::InvalidInput("Missing sections array".to_string())
        })?;
        let citations = input["citations"].as_array().ok_or_else(|| {
            ToolError::InvalidInput("Missing citations array".to_string())
        })?;
        let filename = input["filename"].as_str();

        let output_path = get_output_path(title, filename, "pdf")?;

        // Fallback font detection chain
        let font_family = genpdf::fonts::from_files("/usr/share/fonts/truetype/freefont", "FreeSans", None)
            .or_else(|_| genpdf::fonts::from_files("/usr/share/fonts/truetype/dejavu", "DejaVuSans", None))
            .or_else(|_| genpdf::fonts::from_files("/usr/share/fonts/truetype/liberation", "LiberationSans", None))
            .map_err(|e| ToolError::Upstream(format!("Could not load standard Linux system fonts for PDF generation: {}", e)))?;

        use genpdf::Element;
        let mut doc = genpdf::Document::new(font_family);
        doc.set_title(title);
        doc.set_font_size(10);

        let mut decorator = genpdf::SimplePageDecorator::new();
        decorator.set_margins(15);
        doc.set_page_decorator(decorator);

        // Document Title
        doc.push(genpdf::elements::Text::new(title)
            .styled(genpdf::style::Style::new().bold().with_font_size(20)));
        doc.push(genpdf::elements::Break::new(1.0));

        // Sections
        for sec in sections {
            let heading = sec["heading"].as_str().unwrap_or("Untitled Section");
            let body = sec["body"].as_str().unwrap_or("");
            doc.push(genpdf::elements::Text::new(heading)
                .styled(genpdf::style::Style::new().bold().with_font_size(14)));
            doc.push(genpdf::elements::Break::new(0.5));
            doc.push(genpdf::elements::Paragraph::new(body));
            doc.push(genpdf::elements::Break::new(1.0));
        }

        // References
        if !citations.is_empty() {
            doc.push(genpdf::elements::Text::new("References")
                .styled(genpdf::style::Style::new().bold().with_font_size(12)));
            doc.push(genpdf::elements::Break::new(0.5));
            for cit in citations {
                let id = cit["id"].as_i64().unwrap_or(0);
                let t = cit["title"].as_str().unwrap_or("Unknown Source");
                let link = cit["url_or_doi"].as_str().unwrap_or("");
                doc.push(genpdf::elements::Paragraph::new(format!("[{}] {} — {}", id, t, link)));
            }
        }

        doc.render_to_file(&output_path)
            .map_err(|e| ToolError::Upstream(format!("Could not render PDF document: {}", e)))?;

        Ok(ToolResult {
            content: format!("PDF document successfully generated at: {}", output_path.display()),
            citations: vec![],
        })
    }
}

pub struct CreatePptxTool;

#[async_trait::async_trait]
impl Tool for CreatePptxTool {
    fn name(&self) -> &str {
        "create_pptx"
    }

    fn description(&self) -> &str {
        "Generate a slide deck presentation (.pptx) of the research summary."
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "title": { "type": "string" },
                "sections": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "heading": { "type": "string" },
                            "body": { "type": "string" }
                        },
                        "required": ["heading", "body"]
                    }
                },
                "citations": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": { "type": "integer" },
                            "title": { "type": "string" },
                            "url_or_doi": { "type": "string" }
                        },
                        "required": ["id", "title", "url_or_doi"]
                    }
                },
                "filename": { "type": "string" }
            },
            "required": ["title", "sections", "citations"]
        })
    }

    async fn call(&self, input: serde_json::Value) -> Result<ToolResult, ToolError> {
        let title = input["title"].as_str().ok_or_else(|| {
            ToolError::InvalidInput("Missing title".to_string())
        })?;
        let sections = input["sections"].as_array().ok_or_else(|| {
            ToolError::InvalidInput("Missing sections array".to_string())
        })?;
        let citations = input["citations"].as_array().ok_or_else(|| {
            ToolError::InvalidInput("Missing citations array".to_string())
        })?;
        let filename = input["filename"].as_str();

        let output_path = get_output_path(title, filename, "pptx")?;

        use ppt_rs::{create_pptx_with_content, SlideContent};
        let mut slides = Vec::new();

        // 1. Welcome Slide
        let mut welcome_slide = SlideContent::new(title);
        welcome_slide = welcome_slide.add_bullet("Research Report Presentation Slide Deck");
        slides.push(welcome_slide);

        // 2. Section Slides
        for sec in sections {
            let heading = sec["heading"].as_str().unwrap_or("Untitled Slide");
            let body = sec["body"].as_str().unwrap_or("");
            let mut slide = SlideContent::new(heading);
            for line in body.lines() {
                if !line.trim().is_empty() {
                    slide = slide.add_bullet(line.trim());
                }
            }
            slides.push(slide);
        }

        // 3. Citations Slide
        if !citations.is_empty() {
            let mut ref_slide = SlideContent::new("References");
            for cit in citations {
                let id = cit["id"].as_i64().unwrap_or(0);
                let t = cit["title"].as_str().unwrap_or("Source");
                let link = cit["url_or_doi"].as_str().unwrap_or("");
                ref_slide = ref_slide.add_bullet(&format!("[{}] {} — {}", id, t, link));
            }
            slides.push(ref_slide);
        }

        let output_path_str = output_path.to_str().ok_or_else(|| {
            ToolError::Upstream("Could not convert output path to string".to_string())
        })?;
        create_pptx_with_content(output_path_str, slides)
            .map_err(|e| ToolError::Upstream(format!("Could not generate PPTX deck: {}", e)))?;

        Ok(ToolResult {
            content: format!("PowerPoint presentation successfully generated at: {}", output_path.display()),
            citations: vec![],
        })
    }
}
