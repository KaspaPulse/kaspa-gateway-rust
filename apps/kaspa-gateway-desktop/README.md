# Kaspa Gateway Desktop

Stage 11 desktop runner.

## Commands

Install frontend and Tauri CLI dependencies:

npm install

Run desktop dev shell:

node node_modules/@tauri-apps/cli/tauri.js dev

Build desktop app:

node node_modules/@tauri-apps/cli/tauri.js build

Rust-only validation from repository root:

cargo check -p kaspa-gateway-desktop
cargo test --workspace

## Security posture

- Local static frontend.
- CSP configured in tauri.conf.json.
- Minimal capability using core:default only.
- No shell execution.
- Frontend calls Rust through explicit Tauri commands.
