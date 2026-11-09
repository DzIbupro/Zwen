# 新会话交接：论文阅读改用文件适配路线

## 先读这里

用户已决定放弃旧严格流水线，改为：**导入PDF → 成熟文件输入/提取适配 → 阅读skill → 模型直接生成Markdown → 保存与追问**。

2026-11-09用户已恢复并授权开始开发。当前工作在干净基线创建的 `feat/paper-analysis-v2`，采用Poppler全文文字适配及现有聊天生命周期。最新实现、验证与限制以 `docs/validation/2026-11-09-paper-v2.md` 为准；不要继续旧流水线。

- 仓库：`/home/dzibupro/Zwen/Zwen`
- 当前旧实现与方案记录分支：`feat/paper-analysis`；新实现分支：`feat/paper-analysis-v2`（已从main 9337398创建）。
- 旧实现最后一次代码修复提交：`5f659b2`
- 运行产物：交付时将 `artifacts/zwen-paper-analysis` 更新为v2普通构建，日常主程序 `artifacts/zwen` 保留。以最新验收记录与文件哈希为准。
- 当前方案：`docs/superpowers/specs/2026-11-08-paper-analysis-design.md`
- 当前计划：`docs/superpowers/plans/2026-11-08-paper-analysis.md`
- 用户需求原件：`/home/dzibupro/background/文献阅读skill(2).txt`

## 分支核查与清理决定（2026-11-09）

- 论文分支reflog确认：2026-11-08 03:38:57 +0800，从 `9337398` 创建 `feat/paper-analysis`。截至方案替换提交 `cfcd0f3`，其相对main有16个独立提交。
- 本地 `main`、`origin/main` 与本次GitHub查询的main均为 `9337398cecbcde4d3b2c73d7da53f972afe3629c`；论文提交未进入main或 `feat/desktop-foundation`。
- GitHub分支查询仅返回main与 `feat/desktop-foundation`；PR查询仅返回已合并的桌面功能PR #1，无论文PR。论文工作目前仅在本地专用分支。
- shell直连GitHub因DNS失败；远端状态通过已连接的GitHub工具核查，不把本地远端缓存当实时证据。

用户已允许清除废弃实现或回到论文开发前版本。后续采用**从干净基线重新开分支**：核对main是否有新提交；从确认的基线创建 `feat/paper-analysis-v2`，只带入本交接、新设计和新计划等必要文档，按需更新README。不要整体cherry-pick旧论文代码提交或包含旧说明的完整README/进度记录。

旧分支仅作历史参考，不再继续开发；没有必要reset主分支、强推GitHub或逐个revert旧提交。新分支不继承旧论文源码、测试、依赖、权限与界面接线。确有价值的基础能力应逐项审核后复用，不能把整条旧流水线搬回来。安全要求仍须在新代码中实现，不能因回到旧基线而遗漏。

此前核查阶段只改文档；开发阶段已建立v2分支并实现文字分析路径，旧分支保留历史。Git管理源码与本地PDF/数据库/产物不同：不使用 `git clean -fdx`，不删除用户论文、历史、笔记、密钥；旧产物也不能当新构建运行。

## 为什么要换路线

真实接入qwen3.7-flash后反复失败：JSON围栏、额外闭合符、双栏文字交错、片段ID越批、长文合并预算及严格校验中断。前几次补丁虽通过合成测试，不能代表真实使用已可用。

旧流程强制模型逐批输出EvidenceBatch JSON、抄写长chunkId、经过结构化压缩再输出报告JSON；任何一处不符合协议都可能中断整篇。用户明确认为这超过“上传文件让AI分析”所需复杂度。

用户需求原件中的证据与诚实规则仍然有效，应落实为skill行为和辅助核对，而非把所有定位要求变成阻断整篇的协议。

## 新路线已确定的边界

- 当前选择参考OpenClaw的提取降级方式，实际复用已安装Poppler与Zwen现有聊天；未安装通用agent运行时。原生PDF路径仍待确认。
- 原生PDF仅在服务真实支持时使用；否则本地提取文字，必要时按能力读取页图。
- 全文能放下就直接分析；超限才采用成熟分页/按需读取，不机械切成许多约1 KiB任务。
- 默认Markdown报告；引用无法核对时清楚标注，不让整篇失败。
- 保留有效的服务凭据归属保护、文件权限、停止、保存和净化渲染。
- 保留用户旧PDF、报告、笔记及历史记录；新分析替换默认流程。

## v2当前实现地图

- `crates/core/src/paper.rs`：全文材料、物理页范围、会话源绑定、新库及旧库只读兼容。
- `crates/core/src/paper/SKILL.md`：单一运行阅读规则。
- `src-tauri/src/paper.rs` / `paper_process.rs`：原生文件选择、Poppler适配、保存范围、导出与旧原件。
- `src-tauri/src/chat.rs`：复用原生聊天生命周期，论文会话失败关闭与每轮原文注入。
- `frontend/PaperComposer.svelte`、`frontend/lib/paper-draft.ts`：对话输入区的论文功能与PDF附件草稿；`frontend/App.svelte`、`lib/send-draft.ts`：跨界面截图隔离。
- `scripts/validate_paper_v2.py`：仅合成数据的原生闭环验收。

## 旧实现地图（仅Git历史，不再是当前代码）

- `crates/core/src/papers`：旧存储、版本化解析、预算、提示词、JSON/引用校验。
- `src-tauri/src/papers`：导入/解析、原生任务、模型调用、恢复、追问、导出。
- `frontend/lib/papers.ts`、`frontend/*Paper*.svelte`：论文控制器及界面。
- `crates/transport`：兼容服务请求和SSE。
- `src-tauri/src/secret.rs`：按服务归属的论文钥匙环读取，不能因简化而回退到跨服务误用密钥。
- 旧论文库已有schema2与解析版本快照，迁移前检查实际schema，不能覆盖重建。

旧验收文档在 `docs/validation/2026-11-08-paper*.md`，仅供历史故障与回归参考，不能当新路线成功标准。

## 下一会话第一步

留在当前 `feat/paper-analysis-v2`，先读最新验收记录，不要重复创建分支或带回旧引擎。当前文字闭环已实现。用户后续要求移除独立论文侧栏/页面，改为对话输入区“论文阅读”工具，最新界面验收以 `docs/validation/2026-11-09-paper-chat-composer.md` 为准；普通构建与试用入口以最新验收记录为准。后续优先用户真实论文试读反馈，再按需核实原生PDF、OCR/图表及自动长文工具。此前授权的两次真实服务合成调用已经用完，不能自行追加付费/消耗额度的调试请求。

官方线索（本会话已查阅，实施时复核版本）：
- https://opencode.ai/docs/tools/
- https://docs.openclaw.ai/tools/pdf

## 可直接发给新会话

> 请在 `/home/dzibupro/Zwen/Zwen` 的当前 `feat/paper-analysis-v2` 分支继续。先读 `docs/PAPER-READING-HANDOFF.md` 和 `docs/validation/2026-11-09-paper-v2.md`。文字全文→阅读skill→Markdown与原文追问已实现，旧严格JSON流水线未带入。先核对当前提交和用户试读反馈，保留数据及安全边界，按验收记录中的未完成能力推进，不要重建分支或重做旧方案。
