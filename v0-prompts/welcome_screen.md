# Welcome Screen – Rust + Iced GUI

🧠 Context:
You are building a Rust-native terminal application with a GUI interface using the `Iced` framework. This application aims to mimic modern terminal UIs like **Warp**, including AI integration and interactive elements.

---

🎯 Objective:
Design and implement a **Welcome Screen UI** for the application’s launch state.

This screen should:
- Greet the user with an engaging headline
- Show a brand/logo (placeholder is fine)
- Provide **4 interactive cards or buttons** for core features
- Use a clean, modern layout inspired by terminal tools like Warp, Raycast, or Fig

---

🧱 UI Requirements:

### Layout:
- Fullscreen layout
- Centered main content column
- Logo (top or center)
- Title: `Welcome to AI Terminal`
- Subtitle: `Start with one of these features`
- Feature grid (2x2 cards or buttons), including:
  - 🔍 AI Command
  - 🎨 Theme Switcher
  - 📜 Recent Logs
  - 📚 Open Docs

### Interactivity:
- Each card or button should send a unique message (`Message::LaunchAI`, `Message::OpenLogs`, etc.)
- Should be styled with hover/focus feedback
- Navigable via keyboard or mouse

### Styling:
- Dark theme (black/charcoal background)
- Neon green accent (like terminal cursor)
- Rounded corners
- Subtle shadows
- Monospaced font
- Responsive to screen size (center content always)

---

🔧 Technical Details:

- Use `iced` crate (version `>=0.10`)
- Follow idiomatic Rust patterns:
  - Struct for `WelcomeScreenState`
  - Enum `Message` for all user interactions
  - Modular functions for `view`, `update`, and `style`
- Include a basic placeholder `Logo` component (e.g., a triangle icon or SVG path)
- Use `iced::widget::button`, `column`, `row`, `container`, `text`, and `space`
- All buttons must be wrapped in containers with padding and spacing

---

📤 Expected Output:
- Complete Rust code for `WelcomeScreen` component
- `Message` enum definitions
- `view` and `update` logic
- Styled feature buttons
- Optional: fade-in animation or dynamic welcome message

---

🧪 Bonus Ideas:
- Animated caret in title (`|`)
- Rotating subtitle messages (`"Ready to build?"`, `"Type your first command"`, etc.)
- Theme preview thumbnails on hover
