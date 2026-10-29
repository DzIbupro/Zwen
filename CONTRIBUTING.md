# 开发说明

正式工程：Rust/Cargo workspace（crates/core、src-tauri），Svelte + TypeScript 静态前端。默认 cargo test 测试core与transport，不编译GUI；HTTP测试只绑定回环地址，不访问真实模型。沙箱内运行时需允许本机监听。

## 依赖与构建

本机已验证 Rust/Cargo 1.98.1、Node 24.22.1、pnpm 11.25.0，Fedora 的 WebKit2GTK4.1 构建依赖。前端 Node 仅用于开发/构建，不随桌面程序常驻。不要为此工程重新安装 GTK4/WebKit6 对照依赖。

```bash
pnpm install --frozen-lockfile --store-dir .cache/pnpm-store
pnpm test
pnpm check
pnpm build
CARGO_HOME="$PWD/.cache/cargo" cargo test
CARGO_HOME="$PWD/.cache/cargo" cargo build --release -p zwen-desktop -j 2
./target/release/zwen show
```

开发模式：`scripts/dev-desktop.sh`。它需要 PATH 内有 Node、pnpm 和 Cargo，运行 Vite 与不带 custom-protocol 的调试壳。正式构建默认启用 custom-protocol 并内嵌 dist，运行时无需开发服务器。

浏览器预览：`pnpm dev`，仅监听127.0.0.1:1420。浏览器草稿使用 localStorage，有明确提示；不能据此宣称桌面 SQLite 验证通过。

## 胶囊验证

`python3 scripts/validate_capsule.py "$PWD/target/release/zwen"` 用 smoke 特性构建验证真实窗口尺寸、增长上限、Niri浮动/锚点、模式转移和旧generation失效。只操作该进程拥有的窗口与临时数据库。演示流不是模型集成。

Linux避免设置resizable=false（上游存在200像素最小高度问题）；用相同的min/max约束锁定每次目标尺寸。CSS高度与Tauri逻辑尺寸须按devicePixelRatio/原生scaleFactor换算，不能混用。

## 数据与桌面验证

默认数据目录由 Tauri app_data_dir 决定，Linux 通常为 `$XDG_DATA_HOME/io.github.dzibupro.zwen`（未设置时 `~/.local/share/...`）。`ZWEN_DATA_DIR` 可显式覆盖，主要用于隔离测试。请不要指向现有笔记库或其他应用数据库。

Linux 新建目录权限0700，数据库及已有 sidecar0600；数据库损坏或版本更新时不自动覆盖/重建。草稿state.sqlite3保留原schema1；非敏感模型配置provider.sqlite3使用schema1；首次交付聊天chat.sqlite3使用schema2。三库独立，不存密钥。聊天schema1仅本轮隔离试验，未交付；未知版本拒绝打开并保留文件。Markdown知识库后续实现。

```bash
CARGO_HOME="$PWD/.cache/cargo" cargo build --release -p zwen-desktop --features desktop-smoke -j 2
python3 scripts/validate_desktop.py "$PWD/target/release/zwen"
```

合成验证仅作用于自己启动的窗口与临时数据库；验证自动保存、关闭前追加内容、重载/重启及第二实例 quit。结果写入 git-ignored artifacts/validation。测试特性不会进入普通构建；交付前必须重新构建不带 desktop-smoke 的版本。不要将合成测试二进制作为正式发布版本。

输入法、双屏焦点、长期运行、网络与磁盘故障需分别验收，不能从短测推断通过。未自动修改 Niri 配置或安装全局快捷键。

## 约定

- 所有数据库操作离开 UI 线程，通过受控互斥访问；revision CAS 拒绝过期保存。
- 收起/退出必须先保存，失败保持输入可见；recover 仅保证最后成功保存的数据。
- 预览内容不允许原始HTML、远程图片/脚本；公式错误局部回退。
- 不提交草稿、截图、密钥、数据库、缓存或原始机器日志。
- Provider 只使用用户明确配置的服务与密钥；尚未实现时不伪装生成回答。

项目暂未选择开源许可证；发布或接受外部贡献前由所有者决定。


## Provider和聊天验证

```bash
CARGO_HOME="$PWD/.cache/cargo" cargo test --offline
pnpm test
pnpm check
pnpm build
CARGO_HOME="$PWD/.cache/cargo" cargo build --offline --release -p zwen-desktop --features desktop-smoke -j 2
python3 scripts/validate_provider.py "$PWD/target/release/zwen"
python3 scripts/validate_chat.py "$PWD/target/release/zwen"
python3 scripts/validate_chat_fault.py "$PWD/target/release/zwen"
python3 scripts/validate_chat.py "$PWD/target/release/zwen" --stress 90
```

这些脚本会临时唤出自己启动的窗口；须先保存退出已有Zwen，避免单实例路由到用户实例。全部使用临时目录、合成密钥和127.0.0.1服务，不读取真实草稿。磁盘故障脚本只对临时库加写锁/替换路径，验证RAM恢复和退出保护。测试特性必须在交付前去掉，重新普通构建。

HTTP客户端禁重定向、自动重试与隐式代理，连接/单次读取/总超时为10/30/120秒。文本响应限256KiB、SSE线路限8MiB。请求工作线程拥有SQLite和局部异步runtime，不在GTK线程做I/O；停止会等待终态保存。环境密钥只进入Rust，不通过IPC返回。

首次非敏感配置工具（已有配置保留）：`cargo run -p zwen-core --example configure_provider -- DATA_DIR API_ROOT MODEL KEY_ENV`。不要将密钥值传给这个工具。
