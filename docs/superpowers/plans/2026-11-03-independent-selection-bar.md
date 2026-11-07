# Independent Selection Bar Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development.

**Goal:** 自动选区只显示不抢焦点的小操作条，明确点击后才打开胶囊。
**Architecture:** 复用同进程GTK3与本机gtk-layer-shell，通过一次性短命透明指针进入探针取得近似锚点，再建立小型原生操作条。保留Rust选区ID冻结与后端动作版本校验。主WebView不因自动候选激活。
**Tech Stack:** Rust/Tauri/GTK3，libgtk-layer-shell动态加载；既有Svelte请求界面。
**Spec:** docs/superpowers/specs/2026-11-03-selection-actions-design.md 最新纠正优先。

## Global Constraints

- 鼠标附近坐标是近似锚点，不宣称精确词句边界。
- 不自动调用AI，不上传用户材料，不读取生产库/密钥，不使用截屏OCR或原始输入权限。
- 未点击候选不展示主窗口；无坐标不擅自居中弹胶囊。
- 保留现有草稿、附件、未保存回答和退出保护。

## Review Focus

- 探针最长200ms，首次进入后立即撤销输入区域并销毁；仍需实际输入测试验证不会吞下一次点击。
- 新选区、失焦、关闭菜单、关开监听、退出均应撤销旧操作条及迟到探针。
- 主线程GTK对象不跨线程，回调按当前选区ID和配置revision校验。
- 命令事件重复/迟到不得错发材料或重复请求；页面重载不自动重发。
- 多屏缩放和屏幕边缘要夹紧到显示区域；不可用则报错并保留手动入口。

### Task 1: Native bar and pointer anchor
Files: src-tauri/src/selection_bar.rs (+pure geometry helper if needed), src-tauri/Cargo.toml.
Interface: show(app:&AppHandle,selectionId:u32,revision:u64,actions:Vec<BarAction>,sourcePid:u32), dismiss(app:&AppHandle). BarAction{id,name}; root exports selection::toolbar_action(app,id,actionId,revision). Include close button, other/custom actions/settings, bounded labels, keyboard NONE/exclusive0. Emit diagnostics for desktop-smoke only.
- [x] RED→GREEN geometry/state/lifecycle checks.
- [x] Implement native GTK on main thread with dynamic layer-shell API; fail safely if library/protocol unavailable.

### Task 2: Frontend explicit command routing
Files: frontend/App.svelte, frontend/lib/selection.ts, relevant frontend tests.
Event selection-command {selectionId,actionId,configRevision,commandId}; backend already froze exact ID before event. Handler refreshes inbox, verifies ID and action config revision, then routes existing explicit action flow. Settings event uses existing request-settings. No automatic model on ordinary candidate/refresh/reload.
- [x] RED→GREEN duplicate/stale command and old draft/unsaved answer preservation.
- [x] Main capsule transition occurs only after explicit command; ask stays draft-only.

### Task 3: Backend selection integration
Files: src-tauri/src/selection.rs, src-tauri/src/main.rs, quick_actions.rs helpers, native test driver.
- [x] Valid automatic candidate shows native bar instead of windows::activate.
- [x] Freeze matching ID and validate config revision before handling native command; close stale UI on watcher disable/hide/new source.
- [x] Native model actions emit once only after activating requested surface; no event means no model request.

### Task 4: Verification and delivery
- [x] Frontend test/check/build + relevant Rust suites.
- [x] Native isolated own-app driver: bar present while main hidden, explicit ask/translate/custom/document, cancellation/stale IDs, no prior material leak.
- [x] Test actual input timing on controlled fixture when CUA supports native input; otherwise request bounded user fixture test and state limitations.
- [x] Fresh review, ordinary binary hash, private-branch push, clean only project target.

## Rulings

用户已授权继续开发及本轮近似指针定位验证，无需重复确认执行方式。沿用私有开发分支，子任务按文件所有权分工。未验证的全屏指针探针不得假称无副作用；不改变系统辅助功能设置。

## 验证收尾

前端126、核心/传输127、桌面39（另1钥匙环ignored）通过；原有9项原生场景及新增actions阶段回归通过。定位延迟问题由用户真实输入发现并修补，最终限定场景3拖选/3按下/3点击/3操作条，用户确认定位准确。无所有场景保证；普通程序构建/哈希在development-progress.md记录。
