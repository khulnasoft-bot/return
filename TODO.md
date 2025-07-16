# NeoTerm TODO List

This document outlines remaining tasks and future enhancements for the NeoTerm project.

## High Priority

- [ ] **Refine AI Streaming**: Ensure smooth, real-time streaming of AI responses to the UI.
- [ ] **Contextual AI**: Implement sending relevant block content (selected block, previous command/output) as context to the AI.
- [ ] **AI-driven Command Generation**: Allow AI to suggest executable commands that can be easily inserted into the input bar.
- [ ] **Tool Integration**: Enable AI to use internal tools (e.g., `read_file`, `list_dir`) to gather information or perform actions.
- [ ] **Error Handling**: Improve robust error handling and user feedback for PTY, API, and AI interactions.
- [ ] **Clipboard Integration**: Implement copy-to-clipboard and paste-from-clipboard functionality for text blocks.
- [ ] **File System Abstraction**: Enhance `src/virtual_fs/mod.rs` to truly abstract over different backends (local, cloud, in-memory) rather than just using `tokio::fs` directly.
- [ ] **Plugin System**: Fully implement the `Plugin` trait and `PluginManager` to allow dynamic loading and execution of Lua and WASM plugins with defined APIs and permissions.
- [ ] **Workflow Step Implementations**: Complete `SubWorkflow` and `PluginAction` step types in `src/workflows/executor.rs`.
- [ ] **Settings UI**: Develop a comprehensive UI for all user preferences defined in `src/config/preferences.rs`.

## Medium Priority

- [ ] **Theme Editor UI**: Build a comprehensive UI for live theme editing (colors, fonts).
- [ ] **Keybinding Editor UI**: Create a UI for customizing keyboard shortcuts.
- [ ] **Environment Profile Management**: Implement full CRUD operations for environment profiles.
- [ ] **Command Palette Enhancements**: Add more commands, fuzzy search improvements, and better visual feedback.
- [ ] **Block Actions**: Implement "Export" and "Bookmark" actions for blocks.
- [ ] **Performance Benchmarks**: Implement and integrate the performance benchmarking tools.
- [ ] **AI Tooling**: Expand the `src/agent_mode_eval/tools.rs` with more practical tools (e.g., file system operations, network requests, code execution).
- [ ] **Command History Persistence**: Ensure command history is persisted across sessions.
- [ ] **Theming**: Enhance `src/config/yaml_theme.rs` to support more granular UI element styling and dynamic theme switching.
- [ ] **Error Reporting**: Improve user-facing error messages and provide actionable insights.
- [ ] **Performance Optimization**: Profile and optimize rendering, I/O, and background tasks for large outputs or complex workflows.

## Low Priority / Future Enhancements

- [ ] **Collaboration Features**: Fully implement session sharing and real-time collaboration.
- [ ] **Cloud Sync**: Implement robust cloud synchronization for preferences, history, and workflows.
- [ ] **Drive Integration**: Integrate with cloud storage providers (e.g., Google Drive, Dropbox).
- [ ] **Debugger**: Implement the workflow debugger.
- [ ] **LPC Support**: Integrate the LPC engine.
- [ ] **MCQ Support**: Implement MCQ management.
- [ ] **Natural Language Detection**: Integrate and utilize natural language detection.
- [ ] **Markdown Preview**: Implement a rich markdown previewer.
- [ ] **Syntax Tree**: Integrate and utilize syntax tree parsing.
- [ ] **Asset Macro**: Implement asset macro processing.
- [ ] **WebSocket Server**: Implement the WebSocket server for external communication.
- [ ] **CLI Integration**: Enhance CLI commands for deeper interaction with the running GUI.
- [ ] **Distribution Packaging**: Implement logic for packaging the application for various platforms.
- [ ] **Unit/Integration Tests**: Expand test coverage across all modules.
- [ ] **Documentation**: Add comprehensive inline documentation and user guides.
