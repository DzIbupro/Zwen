# 输入链路与草稿基础 Implementation Plan

> **状态：被v2设计取代，暂停执行。** 本计划按旧Python／纯文本UI范围编写。先完成[技术选型验证](../../decisions/technology-evaluation.md)，再重写实施计划。可复用的存储、选区和截图测试思路保留为参考。

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 交付可恢复草稿窗口，让选区和 Niri 截图可靠进入草稿。

**Architecture:** Gtk.Application 负责单实例与界面；草稿通过独占工作线程的 SQLite 连接持久化；Niri 后端以唯一输出路径关联截图事件。系统输入和状态机独立于 GTK，以便无桌面单元测试。

**Tech Stack:** Python ≥3.11、PyGObject、GTK4、SQLite、unittest；不需要模型密钥。

**Spec:** [设计](../../design.md)；本计划只覆盖设计阶段 A 和 B 的草稿基础，不宣称覆盖完整聊天产品。

## Global Constraints

- Python＋PyGObject＋GTK4＋SQLite，本机原生运行。
- GTK 操作留在主线程；SQLite 连接归一个工作线程所有。
- PRIMARY 3 秒超时；不能静默使用旧剪贴板。
- 同时只允许一个截图请求；命令返回 0 不能当作截图成功。
- 来源未知允许空；点击发送之前不上传选区或图片。
- 不自动修改 Niri 配置；不覆盖 Mod+Shift+E。

## Review Focus

- PRIMARY 无内容、非文本、无效 UTF-8、超过 1 MiB：显示明确错误，不附加旧值。
- 截图事件来自其他路径或已失效请求：忽略，已有草稿保持原样。
- 截图文件尚未可读／损坏／超过 32 MiB：不能成为有效附件。
- 关闭窗口或工作线程写入失败：不静默宣告保存成功。
- 中文输入法组合态与150%缩放：人工验收，不能由单元测试代替。

## 文件与接口

首阶段文件：`paths.py`、`storage.py`、`selection.py`、`capture/state.py`、`capture/niri.py`、`app.py`、`ui/draft_window.py`、`__main__.py`。每个模块随对应任务创建，不预先添加空模块。

### Task 1：草稿持久化与恢复

**Files:** `src/study_assistant/paths.py`、`storage.py`、`tests/test_storage.py`。

**Interfaces:** `Draft(id: str, text: str, source: str | None)`；`DraftStore(database_path: Path)`；`save(draft: Draft) -> None`；`load(draft_id: str) -> Draft | None`；`close() -> None`。存储调用均由同一工作线程执行。

- [ ] 写 `test_round_trip_after_reopen`：保存包含中英文的 Draft，关闭并重新打开数据库，恢复内容相同。
- [ ] 写 `test_missing_draft_returns_none` 与 `test_write_failure_is_reported`：未知ID返回None；不可写路径向上报告异常。
- [ ] 运行 `PYTHONPATH=src python3 -m unittest discover -s tests -p test_storage.py -v`，确认缺少模块导致失败。
- [ ] 实现事务写入、参数化 SQL、schema 版本；数据目录遵循 XDG_DATA_HOME，默认用户本地数据目录。
- [ ] 重跑上述命令，确认案例通过，再提交 `feat: persist and restore drafts`。

### Task 2：可测试的文本输入

**Files:** `selection.py`、`tests/test_selection.py`。

**Interfaces:** `read_selection(primary: bool = True) -> str`；`SelectionError`。返回未经自动strip的文本快照；用stdout字节读取，限定1MiB，超时3秒。

- [ ] 写 mock subprocess 案例：PRIMARY 和 clipboard 参数分别正确；中英文与换行完整保留。
- [ ] 写空值、非零退出、超时、无效UTF-8、超过1MiB案例；断言抛出 SelectionError，且不会发起第二次剪贴板读取。
- [ ] 运行 `PYTHONPATH=src python3 -m unittest discover -s tests -p test_selection.py -v` 确认失败。
- [ ] 实现 `wl-paste --primary --type text --no-newline`；复制入口移除 --primary；读入限制需避免先无限缓存后才判大。
- [ ] 重跑通过，提交 `feat: read explicit text selection snapshots`。

### Task 3：截图状态机与 Niri 事件

**Files:** `capture/state.py`、`capture/niri.py`、`tests/test_capture.py`。

**Interfaces:** `CaptureRequest(id: str, path: Path, status: str)`；`CaptureTracker.begin(path: Path) -> CaptureRequest`；`accept_event(event: dict) -> Path | None`；`invalidate() -> None`。状态：pending、received、expired、cancelled、failed；received仅指收到匹配事件，文件解码成功后才由附件层确认可用。

- [ ] 测试同时发起第二请求被拒绝；匹配路径可接受一次；其他路径、重复事件和失效后迟到事件均返回None。
- [ ] 测试损坏JSON事件不中断后续有效事件处理；命令失败、event-stream断开须使请求失败并保留草稿。
- [ ] 测试图片文件超过32MiB、PNG无效或不可读不进入有效附件；可读文件在上限内再交Gdk解码。
- [ ] 先运行失败，再实现先订阅后请求、UUID绝对路径、事件按行解析和进程清理。实际Esc无取消事件时显示“等待完成／返回草稿”，不将超时描述成用户取消。
- [ ] 运行 `PYTHONPATH=src python3 -m unittest discover -s tests -p test_capture.py -v`，通过后提交 `feat: correlate niri screenshot completion`。

### Task 4：GTK 单实例草稿窗口

**Files:** `app.py`、`__main__.py`、`ui/draft_window.py`、`tests/test_commands.py`、`pyproject.toml`。

**Interfaces:** `main(argv: list[str] | None = None) -> int`；命令：show、ask-selection、ask-clipboard、capture、quit；无命令等于show。命令解析可不导入GTK。

- [ ] 为已知命令、无参数、未知命令写解析测试，确认失败后实现解析。
- [ ] 用 Gio.Application 的 HANDLES_COMMAND_LINE 转交；业务动作与I/O完成回调回到GLib主循环，不在UI线程等待子进程。
- [ ] UI显示摘录、来源输入、附件预览、保存状态；隐藏保留单实例，quit确保关闭工作队列；保存失败提供可重试提示。
- [ ] 添加 console script `study-assistant = study_assistant.app:main`；模块可用 `PYTHONPATH=src python3 -m study_assistant` 启动。
- [ ] 无桌面测试 `PYTHONPATH=src python3 -m unittest discover -s tests -v`；真实Niri会话验证两次启动只有一个主窗口、中文输入、隐藏恢复、重启恢复草稿。
- [ ] 手动完成Firefox和实际阅读器选区、截图保存与Esc、两屏及150%缩放；逐项记录结果，不通过项留为已知问题。
- [ ] 测量默认与Cairo的文本刷新、图片和隐藏PSS，再决定应用默认渲染器；提交 `feat: add single-instance draft workspace`。

## 下一阶段

完成以上验收后，为Provider、SSE解析、消息与附件持久化另写聊天实施计划。首个服务选择和模型配置在那时明确，不预填密钥或默认上传目标。
