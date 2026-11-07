# 桌面基础开发续接记录 · 历史暂停

当前状态：用户已于后续明确说“继续”，暂停解除。本文保留为暂停时快照；完成情况见[桌面基础验证](../validation/2026-10-29-desktop-foundation.md)。

原状态：**用户于本轮要求“等手上的最小任务做完，暂停，等下次说继续再工作”。当前最小的桌面保存/恢复测试已完成。后续只在用户明确说继续后恢复；不安排自动任务。**

## 当前分支与实现

- 仓库：`/home/dzibupro/Zwen/study-assistant`，分支 `feat/desktop-foundation`；基线/远端 main 为 `96fe640645d41e56fe0fb71a24e8397f8e162e52`。
- 实施计划：`docs/superpowers/plans/2026-10-29-desktop-foundation.md`，依据 design.md v3。用户授权下一步后内联开发；不重新讨论已确认的技术选择。
- 已建立正式 Cargo workspace：core + src-tauri；Svelte/TypeScript/Vite 前端。SQLite revision CAS 防止旧保存覆盖；500ms 串行自动保存，失败保留输入并阻止正常关闭；文件用户私有权限。
- 界面：轻面板、草稿、清空/撤销、手动保存、明确标注的 Markdown/表格/公式预览；不伪装成 AI 回答。输入法组合期间 Esc 不收起。
- 桌面：单实例；show/recover/quit；native close 请求前端保存后隐藏；Rust IPC 仅注册四个业务/窗口命令，静态资源 CSP 和 app manifest ACL 已配置。
- 当前 `target/release/study-assistant` 是 **desktop-smoke 特性构建**，不是最终交付二进制。默认 custom-protocol 已启用。合成测试仅在显式环境变量启用时运行；普通发布构建仍需单独生成。
- 没有接真实模型、截图/选区、会话历史、知识整理或知识图谱；没有改 Niri 配置或安装系统启动器。

## 已验证证据

- `cargo test -p study-assistant-core`：10项通过（7存储 + 3 CLI）。原始存储5项、CLI2项曾在桩实现失败，后通过；文件权限回归先失败后通过。
- `pnpm test`：11项通过。原始9项先失败后通过；新增原始HTML覆盖界面、保存中撤回内容误报已保存两项先失败后修复通过。
- `pnpm check`：0错误0警告；`pnpm build`通过。
- native合成验证：实际textarea自动保存；关闭前追加尚未防抖保存的文字，native close握手100ms内隐藏；重新显示并重载后revision=2且完整草稿恢复；第二个进程启动同一隔离数据库也恢复成功。实际表格和KaTeX检查通过，两个进程均退出0。
- 原始数据：`artifacts/validation/{edit,restore}.json`、summary.json、stderr.txt；标准错误为空。短测峰值PSS分别274.79/264.57 MiB，RSS526.6/516.36 MiB；不是60秒稳态或完整产品指标。
- 浏览器预览也验证了编辑、保存、页面重载恢复与格式展示；它使用localStorage，不能当成桌面SQLite证据。
- 尚未关闭Vite后重跑以独立证明离线运行；尚未做最终代码审查、正式无smoke特性发布构建、长期稳定性与完整资源验收。

## 本机工具与存储

- Node不在PATH；本轮复用Codex现有Node 24.22.1：`/home/dzibupro/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node`。
- pnpm：`/home/dzibupro/.cache/codex-runtimes/codex-primary-runtime/dependencies/bin/fallback/pnpm`，实际11.25.0。调用前把上述node/bin加入PATH。
- TypeScript采用6.0.3：Svelte检查器对7要求实验性双版本，因此没有继续堆叠编译器。
- Cargo用 `CARGO_HOME="$PWD/.cache/cargo"`；pnpm store为 `.cache/pnpm-store`。选中依赖已缓存，暂停时暂留以避免重新下载/编译。target约1.4GiB、.cache约394MiB、node_modules约161MiB；后续完成正式交付后清理中间产物，保留可运行程序、源代码与锁文件。
- 未重新安装已清理的GTK4/WebKit6系统包。

## 恢复后按顺序执行

1. 先读本记录、实施计划、git status/log和 `.superpowers/sdd/2026-10-29-desktop-foundation/progress.md`，不要重做已通过的任务。
2. 检查当前Vite服务已停；用现有smoke二进制执行 `python3 scripts/validate_desktop.py "$PWD/target/release/study-assistant"`，补离线证据，随后整理正式validation记录。
3. 按执行技能做一次新上下文只读代码审查；重要问题写回归测试后修复，并全套验证。当前代码尚未审查，不宣称可发布。
4. 更新README/CONTRIBUTING和计划勾选；目前这两份仍主要描述上一轮选型状态。
5. `pnpm build`，再不带desktop-smoke构建正式release；验证CLI单实例/show/recover/quit，保留运行入口到artifacts。避免把测试特性版本当产品发出。
6. 保存界面截图和正式测量结果；完成本地提交并按既有授权同步GitHub（当前新代码尚未推送）。再清理不必要缓存。
7. 向用户展示可交互面板供确认，之后再进入真实Provider的流式问答，不自动把当前应用内容加入上下文。

此次暂停优先于原计划的连续执行；不在暂停后继续审查、构建或开发。
