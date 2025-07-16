# AI Command Bar Component – Rust Project Prompt (Iced)

🧠 Context:
You are building a Rust-native terminal interface (like Warp), and you want to implement an AI-enhanced command input bar. The app uses the Iced UI library for rendering, and integrates with an OpenAI API or local LLM server for intelligent completions.

---

🎯 Objective:
Design and implement a **fully functional AI Command Bar** that:
- Accepts user commands as input
- Sends queries to an AI backend (e.g., OpenAI-compatible endpoint)
- Displays streaming or final response inline
- Shows loading state (spinner/dots)
- Allows command history traversal
- Supports a Matrix-style animation toggle

---

🧱 Functional Requirements:
- Input field (single-line, styled like terminal prompt)
- Submit button (`⏎` icon)
- AI status indicator (loading/spinner/done)
- Response output display (below input)
- Matrix-style toggle (boolean state, toggles animation on terminal bg)
- Keyboard support (Enter to submit, ↑↓ to scroll through history)

---

🔧 Technical Constraints:
- Built using `Iced` (preferred) with Rust
- All UI should be declarative, idiomatic Rust
- Use async API calls with `reqwest` or `ureq`
- Error handling must be graceful with visible messages
- Structure using component-style patterns

---

🎨 UI Style:
- Terminal dark theme
- Input font: monospaced, neon green caret
- Rounded soft edges (inspired by Warp)
- Use animation (if available) for spinner/typing dots
- Responsive to screen resizing (if windowed)

---

📤 Expected Output:
- Full Rust component code (structs + `update`, `view`, and message handling)
- Include async API call example to OpenAI
- Sample model for command history and state
- Include `Cargo.toml` dependencies
- Modular layout preferred (split into `mod ai_command_bar` if needed)

---

🧪 Bonus (if time allows):
- Add suggestions dropdown while typing (from a local Vec<String>)
- Allow right-click context menu with “Rerun”, “Edit”, “Copy to Clipboard”

