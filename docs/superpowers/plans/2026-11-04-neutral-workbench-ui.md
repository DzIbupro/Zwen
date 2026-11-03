# Neutral workbench UI implementation plan

> Use superpowers:executing-plans inline; independent native preference and page-style tasks use dispatching-parallel-agents. TDD for state/behavior; visual CSS via render/interaction checks.

**Goal:** Implement selected workbench design, collapsible sidebar and persisted monochrome light/dark themes.
**Spec:** docs/superpowers/specs/2026-11-04-soft-ui-light-dark-design.md
**Architecture:** Svelte owns UI preference controller and CSS tokens; native appearance.sqlite3 owns durable preferences and GTK theme cache. Initial safe enum-only injection applies theme before main view; browser previews use isolated localStorage fallback. No model data storage changes.
**Tech Stack:** Existing Rust/Tauri/Svelte/CSS/SQLite. No UI framework or remote fonts. Official Lucide SVG assets only for navigation/tool icons; keep logo original.

## Shared tokens/interface

CSS variables --bg,--surface,--surface-soft,--text,--muted,--edge,--accent,--on-accent,--shadow-dark,--shadow-light,--raised-shadow,--inset-shadow,--hover-shadow,--pressed-shadow,--focus-ring,--danger,--danger-soft,--code-bg,--table-head. Light accent #30343c; dark #e4e7ed. No purple or gradients. Main16px, labels14px, bodyline1.65.

IPC load_appearance()->{theme:'light'|'dark',sidebarCollapsed:boolean}; save_appearance({preferences:same})->same. No keys in preferences. window.__ZWEN_APPEARANCE__ bootstraps native UI; runtime GTK theme cache updates only after successful persistence, restyles visible bar without remap. Preview storage key zwen.appearance.v1; never used for production model/draft data.

- [x] Native agent: appearance.rs validated store/default/error tests, initialize cached theme and safe startup injection, wire commands/build/capability; selection bar themed styling/current bar refresh. No frontend files.
- [x] Page agent: tokenizing scoped styles in HistoryPanel/KnowledgePanel/KnowledgeGraph/ProviderSettings/QuickActionSettings/Attachments/SelectionNotice/ActionOutputNotice/RetryNotice only; preserve event/props/IDs/smoke selectors, remove ornamental eyebrow text. No App/global CSS/native edits.
- [x] Root: AppearanceController RED→GREEN load/rapid changes/save failure/roundtrip tests; bridge and early bootstrap. Appearance settings two options and sidebar prefs.
- [x] Root: Sidebar/Icon components, existing function wiring retained, navigation collapse and tooltip/accessibility; rework App markup/globalCSS and Capsule controls, simplify welcome and move demo to secondary menu.
- [x] Root: browser visual/interaction verification light/dark expanded/collapsed, 480px page and fixed480 capsule, Markdown table/formula/errors/retry/attachments. Surface changes/theme changes never generate model calls or discard drafts.
- [x] Root: independent final review; meaningful frontend/native tests, isolated desktop actions/image/selection/appearance persistence; ordinary/no-hooks build, user protected restart, private feature-branch integration and deployment. Clean build cache after verified installation.

Work area: /home/dzibupro/Zwen/Zwen/.worktrees/neutral-ui on feat/neutral-ui. Native managed-worktree API could not identify original repository, so git fallback was used; reflink copy of shared node_modules, CARGO_HOME=/home/dzibupro/Zwen/Zwen/.cache/cargo. Original feat/desktop-foundation unchanged until verified integration.

审查：独立reviewer发现重载偏好竞态和多附件溢出，均修复并复核。完整结果见docs/validation/2026-11-04-neutral-workbench-ui.md。桌面回归使用隔离应用驱动，并不等于CUA真人鼠标验收。普通构建/集成/安装的状态以development-progress最新记录为准。
