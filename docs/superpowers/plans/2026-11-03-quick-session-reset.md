# Quick selection session implementation plan

> Use superpowers:executing-plans task by task.

**Goal:** Import raw selection text, focus at its end, and reset only the current quick task on user close.
**Architecture:** Keep selection import/persistence in SelectionController; scope lifecycle cleanup in App and SendDraftCoordinator. Native toolbar routing remains unchanged.
**Tech Stack:** Svelte, TypeScript, Vitest, Rust/Tauri.

## Unit 1: Raw text import (this heartbeat)
- [x] Add regression tests for candidate and raw multiline text, preservation of existing draft, persistence failure retry without duplication.
- [x] Observe failing tests, remove label only for candidate/raw in selection.ts; retain explicit translate/explain instructions.
- [x] Run frontend tests, type check and production frontend build; commit checkpoint. Do not deploy partially repaired product.

## Unit 2: Caret visibility
- [ ] Inspect capsule mount and retained-window activation; add component tests proving import focuses at end once without stealing caret during editing.
- [ ] Implement explicit import-focus signal; test multiline scroll and late load cancellation.

## Unit 3: Quick task close lifecycle
- [ ] Trace App.leave, selection claim/ack, DraftController and request-ticket restoration.
- [ ] Introduce explicit task ownership/revocation; close stops/saves safely then clears owned state. Late callbacks cannot restore abandoned task or clear new inputs.
- [ ] Test unasked, completed, failed, stopped, late callback, IO error and next selection; expand/screenshot hide preserve content.
- [ ] Run isolated native synthetic regression, ordinary build and deployment only after complete verified lifecycle. Preserve saved history.

## Later UI work
Follow pending soft-ui request; obtain visual layout confirmation before redesign. Inspect native black corners and rounded icon resources separately; do not assume cause.
