# 知识库全选分批整理 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans; continue inline in this chat under the existing authorization.

**Goal:** 全库选择后显式确认，按最多8篇串行整理；保留完成批次、重试剩余并可撤销本轮。
**Architecture:** 全库ID查询独立于200条浏览列表。后端固定配置并拥有任务，持久化轮次及批次归属；前端只呈现状态。逆序整轮撤销使用一个事务。
**Tech Stack:** Rust / SQLite / Tauri / Svelte。
**Spec:** ../../HANDOFF-2026-11-07.md（用户已确认行为，要求短计划后继续）。

## Global Constraints
- feat/desktop-foundation及既有私有PR，不合并main、不强推。
- 选择与确认前不调用模型；每批1–8篇。全库枚举不读取正文。
- 测试仅临时数据与本机合成服务，不读生产库/环境/密钥，不请求真实服务。
- 不强关日常实例；安装与重启生效分开记录。

## Review Focus
- 超过200篇和当前搜索过滤不能截断全选。
- 多批撤销中途失败须整轮回滚，手工归属/锁定/关系证据受保护。
- 重试归属同一轮，失败及未处理材料保留，停止后不提交迟到结果。
- 设置变化不切换本轮模型，重复点击不产生并发任务。
- 渲染器重载仍可查询任务；应用退出不能宣称自动恢复执行。

### Task 1: 全库范围与事务撤销基础
Files: crates/core/src/organize.rs, crates/core/tests/organize.rs。
调查修正：NoteStore::all_ids() 已存在，现有205篇导出测试证明不受200条上限影响。复用该API，不新增同义接口；IPC留到Task 2。
- [x] 核对现有全库ID实现及205篇测试，不改list上限、不重做已有枚举。
- [x] 编写多批逆序撤销测试，观察缺少undo_batches API编译失败；覆盖有效多批、无效中途整批回滚、手工/锁定保护。
- [x] 实现 OrganizeStore::undo_batches(&[String])；单批API复用，重复/空范围拒绝。
- [x] cargo test --workspace --offline：200通过/1既有忽略；181前端、Svelte0/0、Vite通过；独立审查无可操作发现。记录验证与下一步并提交。

### Task 2: 后端轮次及任务所有权
Files: 新轮次持久化/任务模块、organize_job.rs、notes.rs、main.rs、命令权限。
- [x] 以临时库测试轮次范围/批次原子关联/跨重开读取；205篇26批及双向故障回滚通过。
- [ ] 任务协调器启动时恢复中断状态（存储层不等于运行任务恢复）。
- [ ] 增加全库枚举IPC、准备确认配置快照、开始/查询/停止/重试/撤销命令。
- [ ] 后端串行分批，固定ProviderConfig，停止/提交共用锁，重试同轮；重载可查询，进程重启标记中断且需重新确认模型。
- [ ] 合成服务验证首批成功后失败、停止、重试、设置变化、整轮撤销。

### Task 3: 全选、确认与进度界面
Files: frontend/lib/knowledge.ts、bridge.ts、KnowledgePanel.svelte、相关测试。
- [ ] 测试全库选择无模型调用、确认快照/批数、重复点击、进度、停止/失败保留、重试及撤销、迟到回调及重载。
- [ ] 接入后端轮次；移除选择数量上限但保持每批8篇；说明全部范围及发送内容和固定模型。
- [ ] pnpm test / check / build、cargo工作区、临时原生驱动、独立审查。
- [ ] 普通release构建、安装备份、更新证据/进度、ff-only回开发分支并推送，不合并main。
