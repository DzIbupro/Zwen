# 开发说明

Zwen 使用 Rust/Cargo workspace、Tauri 2 与 Svelte/TypeScript。先阅读 [README](README.md) 安装桌面依赖。

## 本地开发

```bash
pnpm install --frozen-lockfile
./scripts/dev-desktop.sh show
```

脚本启动仅监听本机的 Vite 服务，再启动调试版桌面壳。浏览器预览可单独运行 `pnpm dev`，但浏览器预览不能验证原生文件选择、SQLite、钥匙环或 Wayland 行为。

## 验证与构建

```bash
pnpm test
pnpm check
pnpm build
cargo test --workspace
cargo build --release -p zwen-desktop
./target/release/zwen show
```

`cargo test` 默认测试 core 与 transport；`--workspace` 还会覆盖桌面 crate，要求已安装 Linux 桌面开发依赖。HTTP 测试使用回环地址和合成服务，不需要真实 API 密钥。测试期间必须允许本机回环端口监听。

原生集成脚本位于 `scripts/validate_*.py`，需要 `desktop-smoke` 测试构建及适当的图形环境。它们会启动窗口，部分脚本要求特定 Wayland 工具；运行前阅读对应脚本，使用隔离数据目录，避免路由到日常运行的应用或触及真实导出目录。

```bash
cargo build --release -p zwen-desktop --features desktop-smoke
python3 scripts/validate_paper_v2.py "$PWD/target/release/zwen"
```

该论文测试生成临时 PDF 并调用本机合成服务，不访问真实模型。测试结束后，重新构建不带 `desktop-smoke` 的普通版本再使用或分发。

合成界面预览：启动 `pnpm dev`，打开 `/scripts/interface-preview.html?screen=knowledge&theme=light`。`screen` 可选 `home`、`knowledge`、`graph`、`settings`、`paper`，主题可选 `light`、`dark`。内容来自内存夹具，不读取个人笔记或调用模型。

## 目录结构

| 目录 | 内容 |
| --- | --- |
| `crates/core` | 本机存储、会话、论文、知识整理及输入校验。 |
| `crates/transport` | 兼容模型请求、流式响应、图片及工具协议。 |
| `src-tauri` | 桌面窗口、系统交互、凭据、文件适配与 IPC。 |
| `frontend` | Svelte 界面、状态控制器与前端测试。 |
| `scripts` | 开发入口、合成验证与品牌资源生成。 |
| `packaging` | 桌面入口模板。 |
| `docs` | 面向使用者的数据说明与操作文档。 |

## 提交约定

- 从最新 `main` 新建功能或修复分支，保持改动范围清晰。
- 优先复用现有组件、控制器和系统工具；新增依赖说明必要性。
- 针对行为变化运行适当测试；视觉交互需要实际界面检查。
- 模型返回值、外部文档与 Markdown 按不可信数据处理；不得据此扩大发送范围或系统权限。
- 保存失败应保留未保存内容；删除与迁移应保护用户数据。整理不得擅自改写笔记正文。
- 不提交个人论文、真实截图、数据库、密钥、绝对个人路径、缓存、构建产物或原始运行日志。测试只使用合成数据。
- 不使用真实付费模型作为常规测试依赖。报告服务问题时不得附带密钥。

项目尚未指定开源许可证。提交贡献前请先通过 Issue 讨论授权与范围，勿提交无权公开的代码或材料。
