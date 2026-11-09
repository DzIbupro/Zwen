<p align="center">
  <img src="frontend/assets/zwen-logo.png" width="96" height="96" alt="Zwen" />
</p>

<h1 align="center">Zwen</h1>

<p align="center">
  把阅读、提问、论文分析和知识整理放进同一个桌面工作流。
</p>

Zwen 是一款面向 Linux 与 Wayland 的本地优先 AI 研究助手。它使用 Rust、Tauri 2 和 Svelte 构建，将对话、划词提问、截图分析、PDF 精读和 Markdown 知识库连接起来。模型服务由使用者自行配置，资料和会话默认保存在本机。

> **项目状态：早期预览。** 当前主要适配 Fedora + Niri / Wayland，提供源码构建，尚未发布正式安装包。其他桌面环境仍需进一步验证。

## 为什么是 Zwen

研究时真正打断思路的，往往不是“没有答案”，而是在阅读器、聊天窗口、截图工具和笔记软件之间反复切换。

Zwen 把这些动作收拢到同一条路径：

1. 在阅读内容时选中文字或截取画面。
2. 带着上下文直接提问，而不是重新组织一遍问题。
3. 阅读 PDF 时保留页面版面，让公式、表格和图表仍可被理解。
4. 把有价值的回答收藏为可编辑的本地 Markdown，并整理成主题与知识网络。

## 主要能力

| 能力 | 说明 |
| --- | --- |
| 对话与历史 | 流式回答、多轮追问、停止生成、历史恢复，支持 Markdown、代码、表格和公式。 |
| 划词提问 | 将选中文字导入草稿，通过主窗口或轻量胶囊继续提问；内容不会自动发送。 |
| 截图分析 | 使用 `slurp` 和 `grim` 框选画面，附加到草稿后交给已配置的视觉模型。 |
| PDF 阅读 | 上传或拖入单篇 PDF，在同一对话中分析、追问并打开原件核对。 |
| 本地知识库 | 将回答保存为 Markdown，支持搜索、编辑、来源补充与导出。 |
| AI 整理 | 手动整理选中笔记，或在新收藏后自动归纳主题；支持复用栏目和整轮撤销。 |
| 知识网络 | 以可旋转、可缩放的球形关系图浏览栏目、主题、笔记和关系证据。 |
| 个性化设置 | 分类管理模型、凭据、明暗外观、划词行为和快捷操作。 |

## 论文阅读

在对话输入框上方选择 **论文阅读**，或直接将单篇 PDF 拖入对话页。填写研究方向可以增强结果与当前工作的关联；留空也能正常分析。

默认报告关注论文概括、研究问题与思路、方法机制、核心主张与证据、适用边界、研究联系，以及值得回到原文核对的页面。

Zwen 默认把选定页面渲染为整页图像，并附带辅助文字，而不是只用 PDF 文本层。这样可以保留公式、插图和表格的版面关系。单次分析最多选择 20 页，并需要支持图片输入的兼容模型。

页图、文本和请求体积都有限制；超过限制时程序会要求缩小页范围。模型仍可能误读细小公式、复杂图表或引用，研究结论应以原文为准。详见 [使用指南](docs/usage.md)。

## 安装与运行

### 环境要求

- Linux 图形桌面；完整的划词和快捷窗口流程主要面向 Wayland / Niri。
- Rust 工具链，支持 Rust 2024 edition。
- Node.js 24 与项目声明的 pnpm 版本。
- GTK 3、WebKitGTK 4.1、OpenSSL 等桌面构建依赖。
- Poppler 用于 PDF 页图，`wl-paste` 用于选区，`slurp` 与 `grim` 用于截图。
- 在界面中保存密钥需要可用的 Secret Service 钥匙环，例如 GNOME Keyring。

Fedora 可安装主要依赖：

```bash
sudo dnf install gcc-c++ make pkgconf-pkg-config openssl-devel \
  gtk3-devel webkit2gtk4.1-devel librsvg2-devel \
  poppler-utils wl-clipboard grim slurp gnome-keyring
```

### 从源码构建

从仓库页面复制 Clone URL，安装依赖后执行：

```bash
cd Zwen
pnpm install --frozen-lockfile
pnpm build
cargo build --release -p zwen-desktop
./target/release/zwen show
```

首次启动后进入 **设置 → 模型与凭据**，填写兼容 Chat Completions 接口的服务地址、模型名称和密钥。论文整页阅读要求当前模型支持图片输入；普通截图提问使用单独配置的图片模型。

本仓库不包含 API 密钥、模型权重或个人资料。模型服务的可用性、费用和数据政策由使用者选择的服务商决定。

开发模式、测试与目录结构见 [开发说明](CONTRIBUTING.md)。

## 命令入口

```bash
./target/release/zwen show                  # 主窗口
./target/release/zwen capsule               # 轻量胶囊
./target/release/zwen select                # 导入选中文字
./target/release/zwen explain-selection     # 解释选中文字
./target/release/zwen translate-selection   # 翻译选中文字
./target/release/zwen screenshot            # 框选截图
./target/release/zwen quit                  # 保存并退出
```

可以将命令的实际路径绑定到桌面快捷键。应用不会自动修改 Niri 配置；划词和截图只会进入草稿，不会直接发送。桌面入口模板位于 [packaging/zwen.desktop](packaging/zwen.desktop)。

## 技术结构

```text
crates/core      本地存储、会话、论文、知识整理与输入校验
crates/transport 兼容模型请求、流式响应与工具协议
src-tauri        桌面窗口、系统集成、凭据、文件与 IPC
frontend         Svelte 界面、状态控制器与前端测试
scripts          开发入口、合成验证和品牌资源工具
docs             使用与隐私说明
```

## 隐私模型

- 导入 PDF、浏览历史、编辑笔记和绘制知识网络均在本机完成，不会因此调用模型。
- 只有明确发送问题时，问题及相关上下文才会提交给已配置的模型服务。
- 论文追问会继续使用保存范围内的原文和页图；仅文字模式不会发送页图。
- AI 整理会发送所选笔记及有限的已有主题、栏目名称；自动整理仅在用户开启后运行。
- 模型密钥可保存在系统钥匙环，或由启动环境中的指定变量提供；普通设置数据库不保存密钥值，界面也不读回密钥。
- 对话、论文、笔记和截图保存在本机应用数据目录。数据不是额外加密的保险库，请自行保护系统和备份。

完整的数据流说明见 [数据与隐私](docs/privacy.md)。提交反馈前请删除密钥、私人路径、真实对话和受限制材料，并阅读 [安全说明](SECURITY.md)。

## 已知限制

- 当前只完整验证 Fedora + Niri / Wayland，未覆盖所有 Linux 桌面环境。
- 尚无正式安装包、自动更新和跨平台构建。
- 模型输出、引用、公式和图表理解仍可能出错，关键结论必须回到原始材料核对。
- PDF 整页阅读依赖支持图片输入的模型；不包含本地 OCR。
- PDF 页图、文本和请求大小均有限制，超长文档需要按页范围分析。

## 参与贡献

欢迎提交可复现的问题和范围清晰的改进。请使用合成样例描述问题，不要上传 API 密钥、私人数据库、真实论文或含个人信息的截图。开发与提交约定见 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 许可证

项目尚未指定开源许可证。公开可见的源码不等同于授予任意使用、修改或再分发许可；若需要这些权利，请先联系维护者。第三方组件保留各自许可证，图标来源见 [Lucide 说明](frontend/assets/icons/SOURCE.md)。
