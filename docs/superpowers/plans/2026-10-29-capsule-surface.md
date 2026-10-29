# Capsule Surface Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans inline. User explicitly requested updating requirements/design and continuing implementation after agreeing on two modes.

**Goal:** Actual desktop capsule with fixed width, downward bounded growth and state-preserving transition to full view.
**Architecture:** One WebView, one draft controller, one parent-owned response buffer; Rust surface generation guards late resize requests. Niri adapter only targets this process's window, no config edits.
**Tech Stack:** Existing Rust/Tauri/Svelte; no new runtime/library stack.
**Spec:** docs/requirements.md v3; docs/design.md v4.

## Global Constraints
- Default/show opens full; capsule opens compact. Selection/capture integration remains future and must route capsule.
- Capsule defaults480×96 logical pixels, max55% screen height; fixed top anchor; own overflow containers.
- Save before hide/quit/switch; preserve draft and response across surface changes; ignore old generation resize calls.
- Demo clearly labeled non-AI, cancellable; no model/network or automatic screen context.

## Review Focus
- Invalid/oversized/NaN height; very small displays: clamp safely (Task1).
- Resize after switching full: generation rejects it (Task1/3).
- UI growing past limit must scroll answer, keep composer visible (Task2/native test).
- Switching while demo streams must retain partial text and stop control (Task2/native test).
- Niri operation must only affect matching PID/application and must not edit user configuration (Task3).

### Task 1: Surface policy/CLI
Files: crates/core/src/{cli,surface}.rs; crates/core/tests/{cli,surface}.rs.
Interfaces: LaunchAction::Capsule; CapsuleBounds::new(screen_width,screen_height); requested_height(f64); SurfaceState with mode/generation and policy. Add failing tests for capsule CLI, bounds, finite values and stale generation; implement and run cargo test.

### Task 2: Compact frontend
Files: frontend/{App,Capsule}.svelte; frontend/lib/{bridge,surface,demo}.ts; frontend/tests/{surface,demo}.test.ts; app.css.
Interfaces: get_surface/set_surface/resize_capsule; surface-changed event. Native state authoritative; observer sends content height with generation; browser preview uses query parameter only. Shared parent demo stream, fixed composer and bounded answer. Add tests first for sizing and stopping stream; then implement, run test/check/build and browser screenshot.

### Task 3: Native integration
Files: src-tauri/src/{main,windows,niri}.rs; build.rs; capabilities/main.json; native smoke script and harness; README/CONTRIBUTING/validation.
Configure actual native size/decorations/floating, get own Niri ID and output geometry with bounded child calls, keep top-left stable as height grows. Test real capsule initial size, stream growth, scrolling, expansion and retained draft/answer, stale resize ignored; retain foundation close/reload/quit tests. Run review, fix material findings with regressions, build normal binary, update PR and clean intermediates.
