# Hakorune (.hako) — VSCode Language Basics

This minimal extension registers the `.hako` extension as a language in VSCode and provides basic editor behaviors:

- File association for `*.hako`
- Line/block comments (`//`, `/* */`)
- Brackets and auto-closing pairs

Colorization (grammar) is not provided yet. You can temporarily map `.hako` to an existing grammar via workspace settings (see `.vscode/settings.json`).

## How to use (local dev)

1. Open this repository in VSCode.
2. Run "Developer: Install Extension from Location..." and select `tools/vscode/hakorune-language`.
   - Alternatively: `code --install-extension tools/vscode/hakorune-language` (Insiders may require `--force`)
3. Open a `*.hako` file. You should get comment toggling and bracket matching.

## Next steps (optional)

- Add a TextMate grammar under `syntaxes/` for colorization.
- Publish to the marketplace when ready.

