# 新会话交接：论文阅读改用文件适配路线

## 先读这里

用户已决定放弃旧严格流水线，改为：**导入PDF → 成熟文件输入/提取适配 → 阅读skill → 模型直接生成Markdown → 保存与追问**。

当前停在文档准备阶段。用户五小时额度接近耗尽，明确要求不再开发，额度恢复后另开会话实施。不要接着给旧流水线打补丁，也不要把本文视为已实施完成。

- 仓库：`/home/dzibupro/Zwen/Zwen`
- 当前旧实现与方案记录分支：`feat/paper-analysis`；新实现计划分支：`feat/paper-analysis-v2`（尚未创建）。
- 旧实现最后一次代码修复提交：`5f659b2`
- 运行产物：`artifacts/zwen-paper-analysis`，仍是旧路线，不能当新方案测试结果。
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

这次只完成核查和方案更新，尚未切换分支、删除代码或回退。Git管理源码与本地PDF/数据库/产物不同：不使用 `git clean -fdx`，不删除用户论文、历史、笔记、密钥；旧产物也不能当新构建运行。

## 为什么要换路线

真实接入qwen3.7-flash后反复失败：JSON围栏、额外闭合符、双栏文字交错、片段ID越批、长文合并预算及严格校验中断。前几次补丁虽通过合成测试，不能代表真实使用已可用。

旧流程强制模型逐批输出EvidenceBatch JSON、抄写长chunkId、经过结构化压缩再输出报告JSON；任何一处不符合协议都可能中断整篇。用户明确认为这超过“上传文件让AI分析”所需复杂度。

用户需求原件中的证据与诚实规则仍然有效，应落实为skill行为和辅助核对，而非把所有定位要求变成阻断整篇的协议。

## 新路线已确定的边界

- 优先借鉴OpenCode/OpenClaw等成熟文件适配；具体代码/库复用尚未选定、未安装。
- 原生PDF仅在服务真实支持时使用；否则本地提取文字，必要时按能力读取页图。
- 全文能放下就直接分析；超限才采用成熟分页/按需读取，不机械切成许多约1 KiB任务。
- 默认Markdown报告；引用无法核对时清楚标注，不让整篇失败。
- 保留有效的服务凭据归属保护、文件权限、停止、保存和净化渲染。
- 保留用户旧PDF、报告、笔记及历史记录；新分析替换默认流程。

## 当前实现地图（只为审计与复用，不是继续旧流程的指令）

- `crates/core/src/papers`：旧存储、版本化解析、预算、提示词、JSON/引用校验。
- `src-tauri/src/papers`：导入/解析、原生任务、模型调用、恢复、追问、导出。
- `frontend/lib/papers.ts`、`frontend/*Paper*.svelte`：论文控制器及界面。
- `crates/transport`：兼容服务请求和SSE。
- `src-tauri/src/secret.rs`：按服务归属的论文钥匙环读取，不能因简化而回退到跨服务误用密钥。
- 旧论文库已有schema2与解析版本快照，迁移前检查实际schema，不能覆盖重建。

旧验收文档在 `docs/validation/2026-11-08-paper*.md`，仅供历史故障与回归参考，不能当新路线成功标准。

## 下一会话第一步

先按上面的清理决定准备新分支，再读取新设计与计划，核查当前服务的原生文件/上下文能力，并定向比较成熟文件适配实现，选定最短可行路径后实施。不要先跑全套旧测试或请求用户重复说明需求。真实API测试先说明调用范围，避免为排查大量消耗用户服务额度。

官方线索（本会话已查阅，实施时复核版本）：
- https://opencode.ai/docs/tools/
- https://docs.openclaw.ai/tools/pdf

## 可直接发给新会话

> 请在 `/home/dzibupro/Zwen/Zwen` 先从现有 `feat/paper-analysis` 分支读取 `docs/PAPER-READING-HANDOFF.md` 及其中的新设计、新计划，再按交接说明从干净主分支基线新建 `feat/paper-analysis-v2`，只带入新方案文档。新路线已经确定：成熟文件适配 → 阅读skill → Markdown分析与追问，旧严格JSON/片段ID流水线退出主流程。保留现有数据和有效安全边界，按计划实施；不要再修补旧流程。
