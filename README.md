# Study Assistant · 轻量级研究助手

为 Linux 阅读与研究工作流设计的桌面助手，优先支持 Fedora + Niri。

**状态：开发准备阶段。当前仓库包含设计、验证结论和 Python 项目基础，尚无可用聊天界面。**

## 目标体验

- 选中文字后通过快捷键带入草稿，解释、翻译或自定义提问。
- Niri 区域截图直接附加到草稿，补充问题后发送。
- 远程模型流式问答、停止生成、历史与草稿恢复。
- Markdown 知识库、中文搜索、来源回溯。
- AI 自动分类到栏目、关联知识点、增量整理、撤销与人工修正保护。
- 可交互的知识网络：缩放、拖动、搜索、筛选、笔记与证据回溯。
- 后续自动划词浮条；快捷唤出不自动采集当前应用内容。

## 技术选择

最终语言与界面框架待对照验证：Python + GTK4、Rust + GTK4、Rust + 系统 WebView。
现有 Python 包仅作基础与验证基线。数据采用 Markdown 笔记与 SQLite 元数据。
截图使用 Niri IPC 完成事件与唯一文件路径。首版须正常展示 Markdown、表格与数学公式。
不在后台常驻本地大模型、OCR 引擎或向量服务。

## 开发入口

- [已确认需求 v2](docs/requirements.md)
- [完整设计 v2](docs/design.md)
- [技术选型验证计划](docs/decisions/technology-evaluation.md)
- [本机验证记录](docs/validation/2026-10-29-fedora-niri.md)
- [旧实施计划（被 v2 取代）](docs/superpowers/plans/2026-10-29-foundation.md)
- [开发环境与验证](CONTRIBUTING.md)

```bash
python3 scripts/check_environment.py
PYTHONPATH=src python3 -c 'import study_assistant; print(study_assistant.__version__)'
```

环境检查只报告依赖，不安装软件、不读取剪贴板、不请求模型、不打开截图。
当前没有 API Key 的配置要求；模型配置会随 Provider 功能加入。

## 里程碑

0. 同等功能的语言／框架对照，确认低资源与故障恢复，以及UI交互样例。
1. 正常格式的问答闭环：选区／截图、表格公式、流式、多轮、停止、草稿与历史。
2. Markdown 知识库：保存、搜索、附件、来源与人工修改。
3. AI 自动栏目与关联整理：证据、撤销、增量处理与人工修正保护。
4. 知识网络可视化与自动划词浮条。

隐藏 PSS ≤100 MiB、普通聊天 ≤180 MiB 是待验收目标，须包含渲染子进程；原型数据不代表成品指标。稳定性必须经过故障注入和持续运行验收，不能承诺绝对零崩溃。
