# UI & Design System 🎨

This document defines the visual layout, color theme tokens, and typography definitions for the **ResearchXYZ** terminal interface.

---

## 1. Terminal Color Palette (Theme)

ResearchXYZ uses a custom, high-contrast dark theme matching the Antigravity color space. Colors are implemented using `ratatui::style::Color` RGB triplets:

| Variable | Color Representation (RGB) | UI Application |
| :--- | :--- | :--- |
| **`bg`** | `#0b0e14` (Rgb 11, 14, 20) | Terminal background/frame padding |
| **`surface`** | `#11141c` (Rgb 17, 20, 28) | Status bar background, input box background |
| **`border`** | `#262a36` (Rgb 38, 42, 54) | Separators and panel outline boundaries |
| **`text`** | `#d4d4d6` (Rgb 212, 212, 216) | Main readable message text |
| **`text_dim`** | `#6b7280` (Rgb 107, 114, 128) | Timestamps, labels, logs, and inactive indicators |
| **`text_faint`** | `#3c4150` (Rgb 60, 65, 80) | Textarea placeholders and disabled states |
| **`accent`** | `#7dd3c0` (Rgb 125, 211, 192) | Cursor indicators, prompt markers (`›`), active operations |
| **`success`** | `#5fb88a` (Rgb 95, 184, 138) | Completed operations marker (`✓`), success messages |
| **`error`** | `#e2574c` (Rgb 226, 87, 76) | Failed operations marker (`✗`), error printouts |
| **`warn`** | `#eab308` (Rgb 234, 179, 8) | Retrying notices, rate limit warnings (`↻`) |

---

## 2. Panels & Geometry

* **Frame**: The main TUI is wrapped in a full-screen block (`BorderType::Rounded`) to isolate the view from previous terminal scrollback.
* **Layout Grid**:
  * **Rows [Flex, 1, 3]**: The height of the conversation block automatically expands to consume all space not occupied by the status row (1 line) and input block (3 to 6 lines).
  * **Input wrapping**: The prompt block wraps text automatically. Cursor navigation boundaries scale based on the content height.

---

## 3. Typography & Monospace Rules

* **Font Selection**: ResearchXYZ relies on the host emulator's default monospace font settings.
* **Text Formatting**: Standard markdown styling (headers, bold, italics, code blocks) is parsed into `ratatui::text::Spans` styled with bold/reverse color modifiers to maintain high readability in standard ANSI-compatible viewports.
