# Study Assistant · 轻量级研究助手

为 Linux 阅读与研究工作流设计的桌面助手，优先支持 Fedora + Niri。

**状态：开发准备阶段。技术选型已完成；当前仓库包含设计、可复现技术样例与历史 Python 基础，尚无可用聊天产品。**

## 目标体验

- 选中文字后通过快捷键带入草稿，解释、翻译或自定义提问。
- Niri 区域截图直接附加到草稿，补充问题后发送。
- 远程模型流式问答、停止生成、历史与草稿恢复。
- Markdown 知识库、中文搜索、来源回溯。
- AI 自动分类到栏目、关联知识点、增量整理、撤销与人工修正保护。
- 可交互的知识网络：缩放、拖动、搜索、筛选、笔记与证据回溯。
- 后续自动划词浮条；快捷唤出不自动采集当前应用内容。

## 技术选择

采用 **Rust + Tauri 2 + TypeScript/Svelte**，SQLite 元数据与 Markdown 笔记。
本机三候选各三轮，Tauri 样例峰值 PSS 中位数约368MiB；RSS合计约616MiB，二者口径不同，尚不代表成品500MB目标达成。[选型证据与限制](docs/decisions/2026-10-29-technology-selection.md)。
截图使用 Niri IPC 完成事件与唯一文件路径；快捷唤出不采集应用内容。
首版正常展示 Markdown、表格和公式；不常驻本地大模型、OCR 或向量服务。

## 开发入口

- [已确认需求 v2](docs/requirements.md)
- [完整设计 v3](docs/design.md)
- [技术选型验证完成情况](docs/decisions/technology-evaluation.md)
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

0. 已完成语言／框架对照；下一步建立正式工程并确认交互样例。
1. 正常格式的问答闭环：选区／截图、表格公式、流式、多轮、停止、草稿与历史。
2. Markdown 知识库：保存、搜索、附件、来源与人工修改。
3. AI 自动栏目与关联整理：证据、撤销、增量处理与人工修正保护。
4. 知识网络可视化与自动划词浮条。

整套应用正常工作约500MB为当前内存目标，须包含渲染子进程，并分别报告PSS/RSS；原型数据不代表成品指标。稳定性必须经过故障注入和持续运行验收，不能承诺绝对零崩溃。
