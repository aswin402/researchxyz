# Content & Interface Layout 📝

This document outlines the content structures, terminal viewport layouts, and output models for **ResearchXYZ**.

---

## 1. Terminal User Interface (TUI) Layout

The TUI splits the terminal horizontally into three distinct panes:

1. **Conversation Viewport (Top)**:
   * Displays the sequence of queries and responses.
   * Renders streaming content from the reasoning LLM (text blocks).
   * Visualizes active background tool logs in real-time.
2. **Status Bar (Middle)**:
   * A single reversed-color line displaying runtime parameters.
   * Renders: Current active Model (`claude-sonnet-4-6` or similar), total session tokens consumed, and current agent process status (e.g. `Thinking...`, `Searching...`, `Writing Document...`).
3. **Research Input Area (Bottom)**:
   * Outlined textarea viewport for query input.
   * Captures multi-line prompts via standard editor keys.

---

## 2. Textual Messaging & Tone

* **System Prompts**: Structured around rigorous, fact-based synthesis. Instructs the agent to always cite sources.
* **Agent Outputs**: Markdown-formatted textual summaries containing explicit footnote citations (e.g., `[1]`, `[2]`).
* **Tool Status Lines**: Snappy, actionable logs explaining current search queries, crawl destinations, or document parsing stages.

---

## 3. Output Deliverables

Generated research deliverables compiled in `workspace/` or the configured output directory follow clean, professional styling:
* **PDFs**: Paginated research reports complete with a title page, table of contents, footnoted inline citations, and an alphabetical bibliography.
* **Word Documents (.docx)**: Styled briefs containing header/footer fields, table properties, and standardized heading hierarchies.
* **PowerPoint Slides (.pptx)**: Structured deck slides (Title, Problem Statement, Key Metrics, Deep Dive, Bibliography) utilizing high-contrast layouts.
