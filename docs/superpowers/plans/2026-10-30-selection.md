# Explicit Selection Import Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** 用户触发的PRIMARY选区导入现有草稿并打开胶囊，等用户手动发送。
**Architecture:** Rust有界非阻塞子进程读取＋单槽收件箱；Tauri启动路由先读后聚焦；前端保存后确认，重载须人工检查。
**Tech Stack:** Existing Rust/Tauri/Svelte/SQLite; cached libc only, existing wl-paste.
**Spec:** docs/superpowers/specs/2026-10-30-selection-design.md

## Global Constraints
- 只读取PRIMARY，不回退CLIPBOARD、不模拟复制、不自动发送。
- 文本64KiB、读取750ms目标；草稿256KiB，失败保留原稿。
- 不输出正文/子进程stderr；不迁移旧草稿schema；不安装新系统依赖。

## Review Focus
- 子进程提前关闭stdout仍不退出、后代持有管道：不得卡死或泄漏读取线程。
- 草稿加载/保存/确认失败、渲染器重载：不得覆盖或自动重复材料。
- 并发快捷键和迟到完成：不得覆盖未确认材料。
- 冷启动事件早于JS监听：可从收件箱恢复。
- 聚焦前读取，真实PRIMARY跨应用尚须另行验收。

### Task 1: Bounded capture and inbox
**Files:** Create crates/core/src/selection.rs; Modify crates/core/Cargo.toml, crates/core/src/lib.rs, crates/core/src/cli.rs; Test crates/core/tests/selection.rs.
**Interfaces:** read_primary_text()->Result<String,SelectionError>; Inbox::begin(mode)->Option<u32>, finish(id,result), claim()->Option<Item>, acknowledge(id)->bool; LaunchAction::Selection(Mode).
- [x] Write integration tests for exact limit, over-limit, timeout, invalid/empty bytes, failure, inherited pipe and claimed inbox/collision behavior.
- [x] Run `CARGO_HOME=$PWD/.cache/cargo cargo test --offline -p zwen-core --test selection` and observe RED.
- [x] Implement and run tests GREEN, then existing core suite.

### Task 2: Native routing and draft import
**Files:** Create src-tauri/src/selection.rs, frontend/lib/selection.ts, frontend/SelectionNotice.svelte, frontend/tests/selection.test.ts; Modify main.rs/build.rs/capability, frontend/lib/bridge.ts, App.svelte, Capsule.svelte.
**Interfaces:** claim_selection()->Option<Item>, acknowledge_selection(id); selection-ready event; SelectionController receives DraftController and SelectionPort, publishes notice/recovery/busy.
- [x] Test append preserving edits, exactly one import across duplicate loads, no consumption before draft load, size/save/ack failures and reload recovery; observe RED via `pnpm test frontend/tests/selection.test.ts`.
- [x] Implement first-read claim and explicit recovery without automatic retransmission; read before windows::activate in startup and single-instance callback.
- [x] Run frontend tests/check/build and native build.
- [x] Synthetic native fixture verifies capsule, stored append, no model request, error preserves draft, reload recovery. Never read user PRIMARY in tests.
- [x] Independent whole-change review, fix Important/Critical with RED→GREEN; update progress/usage, commit and sync draft PR. Preserve release executable and clean only generated target.

## Ledger
- Pre-flight: Task1 Item/Mode fields consumed verbatim in Task2. Existing draft controller flush serializes edits; native inbox has no database writes. No schema change.
- Execution: existing authorized feature branch and working directory retained to avoid duplicate dependency/build storage; no new worktree. Existing requirements authorize this workflow; no renewed design/permission prompt.

- Task1 RED missing module/CLI, then core selection9/9 GREEN; exit ownership test RED missing is_pending then included.
- Native RED: duplicate trigger took the no-ticket branch and activated the window during first read; the 450ms fake helper + own Niri focus check reproduced it. Fix: ignore duplicate trigger while reading, preserve the first read's activation ownership.
- Native focus GREEN in full cold/warm/failure/burst scenario. Exit RED: recovery fixture invoked quit_app with an unacknowledged item and the app exited0 without a completion event; fix adds native pending guard, not only a UI check.
- Final review: independent reviewer found Important manual-import/discard busy race; two deferred-operation regressions RED (busy false). Serialize refresh around local mutations and replay pending refresh afterward.
- Final: Ruling: reviewer Minor late capture overriding hide/expand is Important here — latest explicit window intent is a core interaction promise, so fix and test both intents; cost of this choice is touching the shared window action queue, covered by foundation/capsule regressions.
- Final: Ruling: real cross-app PRIMARY/compositor/visual/native execution declined by reviewer — own synthetic native tests cover compositor and lifecycle, but real reader/terminal/browser compatibility and final visual acceptance remain unclaimed.
- Final: fixed busy/discard race — 3 deferred refresh/import/discard tests RED→GREEN; full frontend45/45 and Svelte0/0.
- Final: fixed late window activation — native lifecycle RED hidden window remapped → GREEN hidden unmapped/full retained, alongside native cold/warm/recovery cases and foundation/capsule regressions.
- Deferred minors: none after lifecycle finding regraded Important and fixed. Normal release rebuilt without desktop-smoke; no automatic real-service request.
