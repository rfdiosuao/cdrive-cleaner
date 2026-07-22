<p align="center"><a href="README.md">简体中文</a> · <strong>English</strong></p>

# C Drive Cleaner

A high-performance Windows system-drive cleaner built with Tauri 2, a Rust backend, and a Vue 3 frontend. The project combines fast scanning, layered safety checks, backups, and optional local or cloud AI analysis.

## Highlights

- NTFS USN Journal scanning and asynchronous directory traversal
- SQLite-backed MD5 and XXHash fingerprint cache
- Authenticode signature verification through WinVerifyTrust
- Layered allowlists and a weighted safety score
- Automatic backup before cleanup and one-click restore
- Local ONNX inference plus optional OpenAI, Anthropic, DeepSeek, Qwen, or custom APIs

## Cleanup Modules

The application covers temporary files, Windows update caches, browser and social-app caches, large and duplicate files, application migration, AI-assisted migration, scheduled cleanup, and emergency low-disk cleanup.

## Technology

- **Frontend:** Vue 3, TypeScript, Vite, Tailwind CSS, Pinia
- **Backend:** Rust, Tauri 2, Tokio, SQLite, `windows-rs`
- **Safety and performance:** Authenticode, AES-GCM, ONNX Runtime, XXHash

## Development

Requirements:

- Windows 10 or 11
- Node.js 20+
- Stable Rust toolchain

```bash
npm install
npm run tauri dev
```

Build a release package:

```bash
npm run tauri build
```

## Release

The repository includes CI for frontend and Rust checks. Version tags trigger release packaging:

```bash
git tag v0.1.0
git push origin v0.1.0
```

Always review candidates and keep a verified backup before deleting system files.

## License

MIT. See the repository files for the applicable license text.
