# 2026-10-29 技术样例（非产品）

完整结论见[决策记录](../../../docs/decisions/2026-10-29-technology-selection.md)。本目录仅含合成材料；不读取用户屏幕、剪贴板、笔记或密钥。运行会反复显示/隐藏自己的窗口，请在可打断的桌面会话执行。

Python GTK、Rust GTK、Tauri 三候选共享 web 页面。Rust 使用 release；正式九轮结果在 results/*-formal-*，每轮 summary.json、事件与压缩列样本 samples.csv。PSS/RSS 均为完整可追踪进程树；CSV 的 KiB 值除以 1024 得 MiB。summary 为采样峰值，无法保证捕获瞬时峰值。

## 复现

在本目录执行；依赖使用系统包，禁止 sudo pip。先安装所需候选的开发依赖，而不是无差别安装所有栈。Python 候选需要系统 GI、GTK4、WebKit6；Rust GTK 需要 gtk4-devel/webkitgtk6.0-devel；Tauri 需要官方 Fedora 构建依赖（WebKit2GTK4.1 等）。

```bash
python3 make_fixture.py
python3 fetch_frontend.py
CARGO_HOME="$PWD/cargo-cache" CARGO_TARGET_DIR="$PWD/build" cargo build --release --manifest-path rust-gtk/Cargo.toml -j 2
CARGO_HOME="$PWD/cargo-cache" CARGO_TARGET_DIR="$PWD/build" cargo build --release --manifest-path tauri-probe/Cargo.toml -j 2
python3 measure.py python-repeat python3 python_host.py
python3 measure.py rust-repeat build/release/study-spike-gtk
python3 measure.py tauri-repeat build/release/study-spike-tauri
python3 recovery_measure.py
python3 summarize.py
```

运行需真实 Niri Wayland 会话。下载脚本核验锁定 tarball 的 integrity，提取 dist/lib 与许可证；无常驻 Node 服务。fixture 图片与 Tauri 图标由标准库生成。编译缓存约数 GiB，复测后按需删除本目录 build/、cargo-cache/、web/vendor/。

measure.py 的阶段时间是观测时间而非精确首帧；WebKit 全页快照须独立开启 SPIKE_SNAPSHOT=1，不能混入仅某一候选的内存对照。SVG/图片等都是真实组件渲染，snapshot 是额外目视证据，不用于伪造 UI。

recovery_measure.py 仅结束由自己启动的样例之 WebKitWebProcess 子进程；随后宿主主动 reload。保留第一次缓冲竞态导致注入时机错误的结果，正式附加结果为 tauri-renderer-recovery。此脚本只证明重载通路，不包含数据库或草稿恢复。

本目录 CSP/IPC 为固定离线测试页配置，内联脚本与测试命令不能直接复制到正式产品；依赖版本是本轮锁定版本，不等于未来自动更新策略。
