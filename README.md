# Study Assistant · 轻量级研究助手

面向 Fedora + Niri / Wayland 的桌面学习助手，采用 Rust + Tauri 2 + Svelte/TypeScript。

**当前已可运行桌面基础预览：草稿编辑、自动保存、重启恢复，以及 Markdown/表格/数学公式预览。尚未接入 AI，不是完整聊天产品。**

## 现在可以使用

- 输入、清空/撤销和本地草稿保存；500ms 自动保存，保存失败保留输入。
- 正常关闭前先保存，然后收起；再次启动同一程序唤出已有窗口。
- 重启或重新加载界面后恢复最后成功保存的草稿。
- 明确标注的排版预览：表格、行内/块级公式、代码；预览不会请求模型。
- `show`、`recover`、`quit` 入口；不自动采集屏幕、选区或当前应用。

本机编译后的入口（构建产物不提交 Git）：

```bash
./artifacts/study-assistant show
./artifacts/study-assistant recover
./artifacts/study-assistant quit
```

`recover` 重新加载并恢复最后成功保存的内容，可能丢弃尚未保存的输入，供界面失效时使用。`quit` 请求现有界面先保存再退出；保存失败时保留窗口。渲染进程失效时先用 recover，不能保证强制终止前尚未提交的文字。

## 后续产品范围

选区解释/翻译、截图附加到草稿、远程模型流式聊天/停止/历史、Markdown 知识库、AI 自动分类栏目与关联、证据回溯和可视化知识网络。自动划词浮条为后续目标；快捷唤出不自动附加当前应用内容。

上述尚未实现的功能不以示例页面代替验收。目前无 API Key 配置要求。图片附件和外链打开尚未集成；前端不允许回答内容执行 HTML/脚本或在窗口内导航到远程网站。

## 开发与证据

- [需求基线](docs/requirements.md) · [整体设计](docs/design.md)
- [桌面基础实施计划](docs/superpowers/plans/2026-10-29-desktop-foundation.md)
- [桌面基础验证](docs/validation/2026-10-29-desktop-foundation.md)
- [技术选型](docs/decisions/2026-10-29-technology-selection.md)
- [开发说明](CONTRIBUTING.md)

约500MB为整套产品内存目标，测量必须包含子进程并分别报告 PSS/RSS。基础预览的短测不能代表完整产品或长期稳定性。Python 包及 tools/technology-probes 仅保留为历史基线/复现材料。
