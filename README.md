# Zwen · 轻量级研究助手

面向 Fedora + Niri / Wayland，Rust + Tauri 2 + Svelte/TypeScript。[仓库](https://github.com/DzIbupro/Zwen) · [当前进度](docs/development-progress.md)

**当前支持用户配置的兼容文本服务、流式多轮问答、停止、历史与本机恢复，以及完整窗口/轻量胶囊。已通过本机合成服务验证；用户本机智谱glm-4.7-flash已完成一次真实文本调用；真实多轮、停止与图片待验证。显式选区入口已接入；截图附件和知识整理尚未接入。**

## 两种窗口

- `show`/直接打开：完整问答、模型设置与历史。
- `capsule`：480×96逻辑像素左右的小浮窗，默认顶部居中，固定宽度向下扩展到屏幕约55%后内部滚动。
- 同一WebView和草稿，切换窗口保留问答；Niri只调整本应用窗口，不修改系统配置。
- 胶囊“演示”始终明确标注非AI，与发送模型分开；选区入口和今后的截图入口使用胶囊。

## 问答与数据

- 草稿500ms自动保存；发送前保存问题，成功接受后才清空未继续编辑的草稿。
- 继续会话附带至多近期8轮，总正文限256KiB；模型配置改变时阻止旧会话转发，需新建对话。
- 历史每页20条摘要，回答按需读取；支持查看、继续旧会话。
- 流式响应、表格、公式和代码正常展示；断流/HTTP错误保留部分回答，不自动重试或拼接。
- 首片段、过程检查点和终态保存在本机。界面重载可恢复当前进程保留的未保存回答；整个进程重启只能恢复最近成功落盘的部分并标记中断。
- 保存失败保留回答和重试入口，阻止覆盖及退出；正常quit先停止请求并保存。
- 不自动采集当前应用、屏幕或选区。只有明确发送时才把草稿及本会话上下文交给配置的服务。

## 本机使用

```bash
./artifacts/zwen show
./artifacts/zwen capsule
./artifacts/zwen recover
./artifacts/zwen quit
```

构建产物不提交Git。`recover`重载界面，草稿只保证最后一次成功保存；渲染进程失效时不等于已持久化所有输入。

用户已选择[智谱配置](docs/providers/zhipu.md)：文本`glm-4.7-flash`，后续图片`glm-4.6v-flash`。提供密钥时在自己的终端运行：

```bash
./scripts/run-with-key.sh
```

输入不回显，只进入应用进程环境；不创建.env文件，不把密钥发进聊天。配置页面仅保存非敏感地址/模型/环境变量名。浏览器是界面预览，不会读取密钥或连接模型。

## 选区快捷入口

选中文字后，由Niri快捷键直接调用以下入口；不要先打开终端或点击Zwen再触发，以免原应用撤销选区。可将命令绑定到你尚未占用的按键，程序没有自动改动Niri配置。

```bash
/home/dzibupro/Zwen/Zwen/artifacts/zwen select
/home/dzibupro/Zwen/Zwen/artifacts/zwen explain-selection
/home/dzibupro/Zwen/Zwen/artifacts/zwen translate-selection
```

分别导入原选区、添加解释前缀、添加翻译成中文前缀。读取PRIMARY后才唤出胶囊，附加现有草稿并保存，**不会自动发送**。保持正在运行的Zwen可沿用用户本机启动时提供的模型密钥；冷启动也可导入，未配置密钥则不能发送。

选区限64KiB、读取目标750ms；失败不读普通剪贴板、不模拟复制、不覆盖草稿。PRIMARY只反映其所有者提供的文字，不能保证每个应用支持或证明它是刚选中的内容。当前原生集成使用合成wl-paste验证，实际浏览器/阅读器/终端矩阵仍待确认。

未确认导入遇到界面重载时，先检查草稿再决定重新添加或丢弃；不自动重复。待处理材料只在当前进程RAM，整个进程被强制终止不保证恢复；已成功保存到草稿的内容照常恢复。

## 后续范围

区域截图附加草稿、图片模型路由、Markdown知识库、AI自动栏目/关联整理、带证据的可视化知识网络。自动划词浮条后续实现，快捷唤出不附加应用整体上下文。当前不以演示回复充当真实模型验收；外链系统浏览器打开尚未接入。

## 开发与证据

- [需求基线](docs/requirements.md) · [开发方案](docs/design.md)
- [Provider实施计划与评审](docs/superpowers/plans/2026-10-30-provider.md)
- [选区验证](docs/validation/2026-10-30-selection.md) · [Provider验证](docs/validation/2026-10-30-provider-chat.md)
- [胶囊验证](docs/validation/2026-10-29-capsule.md) · [技术选型](docs/decisions/2026-10-29-technology-selection.md)
- [开发说明](CONTRIBUTING.md)

约500MB是完整产品目标，必须包含子进程并分别报告PSS/RSS；短时合成负载不能代表真实服务、后续图谱/截图或长期稳定性。旧Python包和技术对照样例仅作历史基线。
