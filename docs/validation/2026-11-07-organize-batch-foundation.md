# 分批整理：事务撤销基础验证（2026-11-07）

## 范围与证据

仅实现核心多批事务撤销，尚未接入轮次、模型任务、IPC或界面。现有`all_ids`及205篇测试已核对，不重复实现；UI列表仍为200。

- TDD：先新增3条集成测试，观察E0599（undo_batches缺失）；实现后organize集成测试20/20通过。
- `undo_batches_reverses_all_commits_and_keeps_manual_evidence`：两批逆序撤销，手工确认关系及其支撑归属、锁定主题保留，两个审计状态均已撤销。
- `undo_batches_midway_sql_failure_rolls_back_every_batch_and_status`：新批先撤销，旧批删除主题时触发器ABORT；两个批次归属均保留，重开库后可一次性撤销全部，证明状态也回滚。
- `undo_batches_rejects_empty_duplicate_missing_or_undone_scope_without_partial_changes`：非法范围不留部分修改。
- `cargo test --workspace --offline`：200通过/0失败/1既有钥匙环忽略。CARGO_HOME及TARGET使用交接缓存。首次缺dist，经pnpm build解决；受限沙箱HTTP测试绑定127.0.0.1失败，经exec工具批准回环监听后全量通过。
- `pnpm test`：32文件、181测试通过。
- `pnpm check`：0错误/0警告；`pnpm build`通过（既有大chunk提示）。
- `git diff --check`通过。
- 独立子代理审查：无可操作回归。审查者默认cargo无法联网/离线缓存不足，以上测试由主代理以指定缓存执行。审查者提出optional兜底可能吞SQL错误，经源码核对optional()?会传播错误，兜底仅处理无行，不成立。

既有Rust测试警告：notes测试未使用i/meta_from、attachment测试多余mut、organize_job测试未使用text；本轮未新增警告。

## 边界与下一步

只用临时库/合成文本及现有HTTP本机合成服务；未读用户生产库、进程环境或密钥，未请求真实模型或上传材料。没有原生UI/真人鼠标/长期证据；未安装此基础变更。

下一步按计划Task 2持久化轮次与原子关联，再做任务和UI；不能用前端lastBatch或批次ID数组冒充持久化本轮恢复。继续保持feat/desktop-foundation及既有私有PR，不合并main。
