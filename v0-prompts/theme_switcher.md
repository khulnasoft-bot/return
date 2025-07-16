# Theme Switcher – Rust + Iced GUI

🧠 Context:
This component is part of a Rust-native GUI terminal application built with the `Iced` crate. The application includes a welcome screen, AI command bar, and matrix animation toggle.

This prompt defines a reusable **Theme Switcher** component that allows users to dynamically change between multiple color schemes (e.g., Light, Dark, Matrix, Custom).

---

🎯 Objective:
Build a responsive, accessible **Theme Switcher UI** using the `Iced` framework.

### Features:
- A dropdown or segmented control to choose from predefined themes
- Options: `Dark`, `Light`, `Matrix`, `Custom`
- Optional live preview area (theme colors displayed visually)
- State persists within the application session
- Sends `Message::ThemeChanged(ThemeVariant)` on change

---

🧱 UI Design Requirements:

#### Component Layout:
- Title/Header: `Choose Your Theme`
- Select control (Dropdown or radio/segmented buttons)
- Optional preview pane with background & font color samples
- Apply button (if you want confirmation-based selection)

#### Themes:
\`\`\`rust
enum ThemeVariant {
  Dark,
  Light,
  Matrix,
  Custom,
}
