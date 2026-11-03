# Retry and short selection implementation plan

> Use superpowers:executing-plans inline; independent short-selection diagnosis delegated under dispatching-parallel-agents.

**Goal:** Restore failed request drafts, automatically retry pre-text temporary failures, and diagnose short-selection misses without pretending synthetic anchors are real pointer tests.
**Architecture:** A cancellable Rust transport wrapper reuses immutable messages and one native logical turn. SendDraft retains recovery tickets until terminal completion. Native PRIMARY/probe diagnosis remains separate.
**Tech Stack:** Rust/reqwest/tokio, Tauri, Svelte/TypeScript/Vitest, GTK fixture.

- [x] Add failing retry policy/HTTP tests; implement timeout/temporary429 classification, Retry-After and capped exponential delay with cancel.
- [x] Integrate native single-turn status publication and defer screenshot consumption until success. Keep logical action ID so document auto-save remains valid.
- [x] Add RED→GREEN SendDraft partial-failure and selection-prompt restoration tests, protect newer edits and closed tasks.
- [x] Show retry status/countdown plus stop; add manual restore-question control after terminal failure, preserving partial history and newer draft text.
- [x] Run synthetic 429→success, timeout→success, permanent429, stop-during-delay, partial→failure and screenshot payload preservation.
- [x] Diagnose short selections using actual short synthetic text; obtain human pointer evidence before changing production probe logic. Record unsolved cases honestly.
- [x] Review, ordinary build/no-hook check, install after user saved/exited; save next progress and clean generated build cache.
