VSCode — Hakorune (.hako) Quick Setup

Two minimal options to get started while upstream registrations land.

Option A — Workspace mapping (fastest)
- The repo includes `.vscode/settings.json` with:
  - `"files.associations": { "*.hako": "javascript" }`
- This yields basic colorization via JS grammar immediately.

Option B — Local language basics extension (recommended)
- Folder: `tools/vscode/hakorune-language/`
- Provides:
  - Language registration for `.hako`
  - Comment toggling and bracket pairs via `language-configuration.json`
- Install locally:
  - VSCode: "Developer: Install Extension from Location..." → select the folder
  - Or CLI: `code --install-extension tools/vscode/hakorune-language`

Next steps
- Add a TextMate grammar under `syntaxes/` for true colorization.
- Publish the extension when stable; replace workspace mapping.

