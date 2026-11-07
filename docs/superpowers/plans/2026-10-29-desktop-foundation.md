# Desktop Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement task-by-task. The user authorized the next development step after accepting the selected architecture; execute inline.

**Goal:** Deliver a real Rust/Tauri + Svelte desktop panel with SQLite draft recovery and readable Markdown/math, ready for subsequent Provider integration.

**Architecture:** Rust owns a revision-checked SQLite draft. Svelte owns presentation and a serialized autosave controller; Tauri exposes only named business/window commands. Native close requests save before hiding, and CLI show/recover/quit supports Niri without reading screen context.

**Tech Stack:** Rust, Tauri 2.12.5, TypeScript/Svelte, Vite, rusqlite, Marked, DOMPurify, KaTeX.

**Spec:** docs/design.md (v3), docs/requirements.md, docs/decisions/2026-10-29-technology-selection.md, first next-step item only.

## Global Constraints

- Fedora/Niri/Wayland; no automatic capture or context upload.
- Approximately 500 MB whole-app target; report PSS/RSS separately, do not call a prototype a complete product.
- Draft debounce at most one second, explicit flush before hide/quit; save failure keeps content and window available.
- No API key or real Provider required for this foundation. Preview must be explicitly labeled; no simulated answer passed off as AI.
- Static local frontend; no Node runtime in release. Keep only selected dependencies and avoid duplicate toolchains.

## Review Focus

- Rapid edits during an in-flight save must preserve the latest text (Task 2).
- Stale or duplicate requests must not overwrite committed text (Task 1).
- Corrupt/newer SQLite must fail without truncating user data (Task 1).
- Disk/save failures must not hide/quit or erase the draft (Task 2).
- Markdown injection, code containing dollar signs, malformed and oversized formulas must remain readable and inert (Task 2).

## Task 1: SQLite draft core

Files: Cargo.toml; crates/core/Cargo.toml; crates/core/src/lib.rs; crates/core/tests/drafts.rs.
Interfaces: `Draft { text: String, revision: u64 }`; `Store::open(&Path) -> Result<Store, Error>`; `load(&self) -> Result<Draft, Error>`; `save(&mut self, expected_revision: u64, text: &str) -> Result<Draft, Error>`. Limit text to 256 KiB UTF-8. Single main draft for this milestone.

- [x] Write integration tests for empty database, Unicode/SQL text roundtrip across reopen, independent writers/conflict, oversized rejection preserving old text, newer schema/corrupt-file preservation.
- [x] Run `cargo test -p study-assistant-core` against stubs; observe behavioral failures.
- [x] Implement SQLite schema v1, FULL synchronous WAL, bounded busy timeout, transaction + compare-and-swap revision; no destructive repair.
- [x] Run core tests and commit.

## Task 2: Frontend draft and content

Files: package.json; pnpm-lock.yaml; vite.config.ts; tsconfig.json; index.html; frontend/main.ts; frontend/App.svelte; frontend/app.css; frontend/lib/{draft,render,bridge}.ts; frontend/tests/*.test.ts.
Interfaces: asynchronous `DraftPort.load/save`, snapshot callback exposing text, status, error; controller edit/flush; safe `renderMarkdown(text): string`. Bridge uses Rust in desktop, clearly labeled browser-only local preview otherwise.

- [x] Write Vitest tests for rapid input during save, persistence retry, no hiding after flush failure, restoration, Markdown/table/math/code and injection.
- [x] Run tests against empty implementations and confirm expected failures.
- [x] Implement serialized 500ms autosave with revision checking; never silently resolve conflicting or failed saves. Build light Chinese UI with draft, labeled format preview, save/hide/quit and keyboard hints; keyboard Escape respects IME composition.
- [x] Run frontend suite, Svelte/TypeScript check and Vite build; commit.

## Task 3: Desktop integration and evidence

Files: src-tauri/{Cargo.toml,build.rs,tauri.conf.json,capabilities/main.json,src/main.rs}; src-tauri/tests/smoke.js; scripts/dev-desktop.sh; docs/validation/2026-10-29-desktop-foundation.md; README.md; CONTRIBUTING.md.
Interfaces: load_draft/save_draft + hide_window/quit_app, minimal capabilities; single-instance CLI show/recover/quit. Renderer state reloads from SQLite. No automatic renderer restart loop.

- [x] Add CLI parsing tests for explicit show/recover/quit and unknown input; run RED then implement.
- [x] Wire commands off UI thread and restrict them to main window; native close emits save-before-hide request. Register custom-command ACL in build manifest.
- [x] Build release, run native smoke against isolated synthetic SQLite: actual textarea autosave, rich preview, close/hide and show, renderer reload and process restart restoration. Smoke instrumentation compiled only under a dedicated feature.
- [x] Exercise actual UI in browser preview, capture screenshot, measure native whole process tree; distinguish browser preview from desktop SQLite results.
- [x] Run all checks, fresh whole-change review, address material findings with regression tests. Update docs/status, commit and sync repository. Remove intermediate build/download caches as needed while retaining runnable release and source/locks.
