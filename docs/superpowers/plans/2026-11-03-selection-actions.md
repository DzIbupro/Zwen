# Selection Actions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 用户划词后在胶囊中选择提问、翻译、解释或自定义提示词，并可将完整结果保存为Markdown文档。
**Architecture:** 扩展现有PRIMARY收件箱与胶囊。动作配置在本机持久化，选择动作时冻结选区，独立文本请求不携带旧上下文/附件；完整成功结果先收藏再导出，失败独立重试。
**Tech Stack:** Rust、Tauri、Svelte、TypeScript、SQLite；不新增常驻服务。
**Spec:** docs/superpowers/specs/2026-11-03-selection-actions-design.md（用户已确认）。

## Global Constraints

- 不自动请求真实模型或上传材料；明确点击动作才发送选中文字。
- 默认文档目录 ~/Documents/Zwen，可修改，内部知识库根目录不变。
- PRIMARY支持依赖源应用；固定胶囊位置，不宣称检测所有应用鼠标松开。
- 已有草稿/附件/进行中请求不得被覆盖或混入快捷动作。

## Review Focus

- 连续选区、动作已领取后再次划词：候选可替换，已冻结的材料不可替换。
- 请求失败、停止和本机回答保存失败：均不可自动输出完整文档。
- 文档重试、文件重名、输出目录不可写：不覆盖，不重复生成笔记或请求AI。
- 新旧请求迟到事件：只匹配本次requestId。
- 配置损坏或并发保存：报错保留用户输入，不默默改成默认。

### Task 1: 动作配置与文档输出
Files: src-tauri/src/quick_actions.rs; frontend/lib/quick-actions.ts; frontend/QuickActionSettings.svelte; 对应隔离测试。
Interfaces: QuickAction {id,name,prompt,output:'display'|'document',enabled}; QuickActionConfig {revision,actions,outputDirectory}; IPC load_quick_actions,save_quick_actions(expectedRevision,actions,outputDirectory),save_action_document(requestId,selection,actionId,actionName,outputDirectory) -> {noteId,path}。
- [x] 新增失败测试：配置校验/版本冲突/损坏，输出仅complete且saved请求、重复重试/同名文件/写入失败。
- [x] 实现并运行对应Rust/TS测试。

### Task 2: 选区候选更新与独立请求
Files: crates/core/src/selection.rs; src-tauri/src/selection.rs; src-tauri/src/chat.rs; 对应隔离测试。
Interfaces: candidate claim只预览不冻结；freeze_selection(id)原子校验ID并冻结；start_selection_chat(requestId,prompt)独立纯文本请求（无旧上下文/截图）。
- [x] 先失败测试：新候选替换、冻结候选不可替换、旧ID拒绝、已有手动导入不被自动候选覆盖。
- [x] 实现防抖、过滤自身应用选区；独立聊天入口及无旧附件测试。

### Task 3: 菜单与应用串联
Files: frontend/SelectionNotice.svelte; frontend/Capsule.svelte; frontend/App.svelte; frontend/lib/chat.ts; frontend/lib/bridge.ts; frontend/lib/selection.ts; src-tauri/src/main.rs; 对应测试。
- [x] 测试冻结动作、旧草稿保护、独立发送、完整回答后保存、失败重试不重发。
- [x] 四项菜单与其他动作列表；设置集成；UI在缺少模型时保留选区。
- [x] requestId绑定动作快照，完成后单次保存文档，错误提供重试。

### Task 4: 验收与交付
- [x] pnpm test / pnpm check / pnpm build；cargo test --offline --release；desktop测试。
- [x] 隔离原生驱动串联菜单→翻译/解释→设置→文档输出，不使用生产库或真实API。
- [x] 新审查修补；普通版重建，哈希及无smoke标记核对；保存验证文档，推送私有开发分支并清理项目target。

## Rulings

- 用户已确认方案并要求实现，继续沿用现有私有开发分支，避免再要求重复确认执行方式。
- 子任务限制文件所有权，根协调器负责主程序注册、桥接与集成。

## 完成记录

前端111、核心/传输126、桌面26（1钥匙环忽略）通过；新增actions与原有selection/chat/knowledge原生隔离驱动通过；两项P2修补复审通过。普通版已更新，无smoke标记。CUA原生能力缺失，跨应用真实鼠标验收仍不宣称通过。
