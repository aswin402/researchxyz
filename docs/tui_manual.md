# ResearchXYZ — TUI Manual 🖥️

This guide describes how to navigate, interact, and control the **ResearchXYZ** Terminal User Interface (TUI).

---

## 1. Interface Panels

The screen is divided horizontally into three main areas:

1.  **Conversation Viewport (Top)**:
    *   Fills the upper area of your terminal.
    *   Displays user prompts, streaming text chunks from the LLM, log outputs, and active tool indicators (e.g. searching, scraping, or reading files).
2.  **Status Bar (Middle)**:
    *   A single, reversed-color row highlighting active status metrics.
    *   Shows: Active Model name, token counts, and current agent operation status.
3.  **Research Input Area (Bottom)**:
    *   An outlined input text box where you can type multi-line research queries.

---

## 2. Keyboard Control Bindings

| Key Combination | Action |
| :--- | :--- |
| **`Ctrl+Enter`** | Submit the prompt to start research. |
| **`Esc`** | Exit the application and restore terminal raw settings. |
| **`Backspace` / `Delete`** | Edit prompt characters. |
| **`Left` / `Right` / `Up` / `Down` Arrow keys** | Move cursor inside the input area. |

---

## 3. Visual Status Codes

Tool execution rows are rendered with distinct state symbols:
*   `⠋` (Teal / Dim): Tool request is currently running in the background.
*   `✓` (Green): Tool completed successfully.
*   `✗` (Red): Tool encountered an error (HTTP timeout, parsing failure, etc.) and returned it to the agent core.
*   `↻` (Amber): Tool is rate-limited and backing off before retrying.
*   `📁` (Green) -> URL: A PDF, Word doc, or slide deck has been successfully compiled and written to the output directory.
