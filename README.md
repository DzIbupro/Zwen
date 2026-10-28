# Study Assistant · 轻量级研究助手

为 Linux 阅读与研究工作流设计的桌面助手，优先支持 Fedora + Niri。

**状态：开发准备阶段。当前仓库包含设计、验证结论和 Python 项目基础，尚无可用聊天界面。**

## 目标体验

- 选中文字后通过快捷键带入草稿，解释、翻译或自定义提问。
- Niri 区域截图直接附加到草稿，补充问题后发送。
- 远程模型流式问答、停止生成、历史与草稿恢复。
- 收藏、标签、中文搜索、来源回溯、Markdown 导出。
- 后续可选：自动划词浮条、带证据的概念关系图。

## 技术选择

Python + PyGObject + GTK4 + SQLite；网络优先使用 libsoup 3 异步 API。
截图使用 Niri IPC 完成事件与唯一文件路径。Cairo 为轻量渲染候选，保留性能验收。
不在后台常驻本地大模型、OCR 引擎或向量服务。

## 开发入口

- [完整设计](docs/design.md)
- [本机验证记录](docs/validation/2026-10-29-fedora-niri.md)
- [第一阶段实施计划](docs/superpowers/plans/2026-10-29-foundation.md)
- [开发环境与验证](CONTRIBUTING.md)

```bash
python3 scripts/check_environment.py
PYTHONPATH=src python3 -c 'import study_assistant; print(study_assistant.__version__)'
```

环境检查只报告依赖，不安装软件、不读取剪贴板、不请求模型、不打开截图。
当前没有 API Key 的配置要求；模型配置会随 Provider 功能加入。

## 里程碑

1. 输入链路与可恢复草稿：GTK 单实例、选区、Niri 截图、附件状态。
2. 聊天闭环：一个 Provider、流式、多轮、停止、历史与错误恢复。
3. 研究整理：收藏、标签、中文搜索、来源与导出。
4. 自动浮条与概念图：在真实使用反馈后推进。

隐藏 PSS ≤100 MiB、普通聊天 ≤180 MiB 是待验收目标，原型数据不代表成品指标。
