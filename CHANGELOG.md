# ResearchXYZ Changelog 📝

All notable changes to the **ResearchXYZ** project will be documented in this file.

---

## [v0.0.1] - 2026-06-19

### Added
- **Interactive Configuration Wizard (`researchxyz configure`)**:
  - Implemented interactive CLI setup using the `inquire` library.
  - Allows selecting LLM provider (Anthropic, OpenAI, DeepSeek, Groq, OpenRouter, Google AI Studio, or Auto).
  - Prompts for model names with intelligent defaults, masked API key inputs, and custom API base URLs.
  - Saves the resulting settings immediately to `~/.config/researchxyz/config.toml`.
- **In-TUI `/model` Selection Menu**:
  - Intercepts `/model` input query in the main prompt area to open a centered visual pop-up dialog box overlay.
  - Uses key listeners when active:
    - `Up` / `Down` Arrow keys to navigate providers and models.
    - `Enter` key to select/confirm the active option and advance steps.
    - `Esc` key to cancel selection and close the menu.
  - Dynamically updates active `llm.provider` and `llm.model` configurations in-memory and saves them to the disk configuration file for immediate, persistent application.
- **TUI Modal Draw Engine**:
  - Added `centered_rect` calculations and `Clear` block layout drawing inside `src/tui/draw.rs` to render visual menu overlays on top of the main panels.
  - Styled with the custom Antigravity Teal accent palette.

### Documented
- Documented TUI key bindings and `/model` navigation in [docs/tui_manual.md](docs/tui_manual.md).
- Added quick config wizard setup instructions in [docs/config_guide.md](docs/config_guide.md).
- Updated [README.md](README.md) with quick start examples for both TUI and configure modes.
