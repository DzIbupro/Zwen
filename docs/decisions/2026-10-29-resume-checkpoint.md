# 技术选型续接记录 · 用户要求暂停

历史状态：暂停已于2026-10-29由用户“继续技术选型”解除。**本文件保留为历史快照，下述待办/暂停状态不再有效。** 完成情况见[选型决策](2026-10-29-technology-selection.md)。

原记录： 用户额度不足，要求完成一小步后停止；本次完成的是保存可恢复进度并停止自己的后台编译。不要自动安排继续任务或自动化。

## 最新需求（覆盖旧预算）

- 整套软件内存目标约500MB，略接近／上下浮动可接受；避免1GB、1.5GB、2GB级常态占用。旧100／180MiB不是用户当前限制，后续更新需求、设计、README与选型计划。
- 保留所有v2功能：Vida式漂亮轻面板、显式选区／截图、正常Markdown／表格／公式、远程模型问答、MD知识库、AI自动分类栏目和关联、可视化知识网络。快捷唤出不采集当前应用内容。
- 稳定性与合理资源占用优先；不因用户不熟悉Rust而排除它。不能声称绝对零崩溃。
- 用户另要求：**选型完成后，清理未采用方案的新增依赖与下载／编译产物，节省磁盘。** 现在保留可续编缓存，避免下次全部重下；最终需执行清理并报告释放空间。

## 正式仓库

本地 `/home/dzibupro/Zwen/study-assistant`；远端 `https://github.com/DzIbupro/study-assistant`，private，main HEAD `3bb543b5c64a2c0363021ce8cbf9b408fe4bb014`。GitHub App已安装，仅授权这个仓库；连接器可创建tree/commit/update_ref。最近检查工作区干净（本续接文件除外）。新选型样例在仓库外，不是产品实现。

## 已完成

1. 查阅Tauri与gtk-rs官方构建要求。
2. 用户在终端安装官方Fedora依赖：`rust cargo gtk4-devel webkitgtk6.0-devel webkit2gtk4.1-devel openssl-devel libappindicator-gtk3-devel librsvg2-devel libxdo-devel`。
3. 工具链：rustc/cargo 1.98.1；GTK4 4.24.1；WebKitGTK6和4.1均2.54.1；Python GI可用WebKit6。
4. 验证目录 `/home/dzibupro/Zwen/technology-spike-2026-10-29` 已准备三候选：Python GTK4+WebKit6、Rust GTK4+WebKit6、Rust Tauri（GTK3+WebKit4.1）。
5. 同一离线网页：真实Marked Markdown、DOMPurify、KaTeX公式、Cytoscape 50节点图谱、合成1280×720 PNG、50段模拟流式更新、损坏公式与脚本净化检查、20次图谱重建、100次显隐。无需模型密钥，无用户材料上传。
6. npm官方下载已核验integrity；锁记录 `web/vendor-lock.json`。版本：marked18.1.2、dompurify3.4.17、katex0.19.1、cytoscape3.34.4。
7. Python功能烟测 `results/python-gtk-1/` 完成：table/formula/image/graph/code均true、ticks50、损坏公式与脚本净化recovered=true、100次显隐，退出0，总约36秒。
8. 此次Python树合计初测：峰值PSS453.3MiB、RSS738.4MiB；隐藏PSS424.4MiB。**当时Rust在编译，只有一次运行，且含20次图谱重建；不能作为正式公平资源排名或成品承诺。** 这是有真实WebView的数字，不能继续拿旧40MiB纯文本原型代表完整UI。

## 编译进度和已定位问题

- 共享缓存环境：
  - `CARGO_HOME=/home/dzibupro/Zwen/technology-spike-2026-10-29/cargo-cache`
  - `CARGO_TARGET_DIR=/home/dzibupro/Zwen/technology-spike-2026-10-29/build`
- Rust GTK4固定gtk4=0.11.5、webkit6=0.6.1。第一次构建失败：webkit6引用gtk::Accessible，而gtk4这版该类型需v4_10 feature。已在`rust-gtk/Cargo.toml`添加`features=["v4_10"]`；重试当时在等待共享build锁，**修复尚未通过构建验证**。日志`results/rust-gtk-build.log`、`rust-gtk-build-retry.log`。
- Tauri固定tauri2.12.5、tauri-build2.7.4。构建已推进到wry/tauri-runtime等，用户暂停时尚未完成。日志`results/tauri-build.log`。
- 已对仅属于本次样例的两个cargo构建进程发送SIGINT，PID记录`results/pause-checkpoint.json`；不能宣称两个Rust候选可运行。
- 尚未调试的潜在接口细节：GTK Rust evaluate_javascript签名；Tauri generate_context可能需要默认icon；Tauri本地资产与IPC权限。遇到报错按日志定位，不把构建问题直接算作架构不稳定。
- `python_host.py`在首轮完成后新增WebKit snapshot保存步骤，尚未再跑。正式公平资源对照应统一关闭截图采样或单独做视觉验证，避免截图增加某一候选内存。

## 继续时执行顺序

1. 读取本记录；确认用户已要求继续。先检查后台是否停止、git与磁盘状态，不重做已安装依赖与npm下载。
2. 完成两个Rust release构建（共享target会串行，不要无谓重复启动多个）。命令在验证目录：

```bash
CARGO_HOME="$PWD/cargo-cache" CARGO_TARGET_DIR="$PWD/build" cargo build --release --manifest-path tauri-probe/Cargo.toml -j 2
CARGO_HOME="$PWD/cargo-cache" CARGO_TARGET_DIR="$PWD/build" cargo build --release --manifest-path rust-gtk/Cargo.toml -j 2
```

3. 用`measure.py <label> <command>...`逐一运行，先修复功能错误，再在无编译干扰时每候选重复3次。运行时需桌面会话访问（sandbox默认不能连接，exec require_escalated已有授权）。GTK候选同设置GSK_RENDERER=cairo；Tauri GTK3与WebKit4.1不同，清楚注明。
4. `measure.py`汇总启动样例及其后代PSS/RSS、进程数、CPU ticks和阶段事件，日志在results。其默认窗口约36秒、隐藏16秒，**尚未满足原计划60秒隐藏、30分钟稳定性或8小时发行验收**。需决定并记录本轮选型与发行验收的合理边界，不能夸大。
5. 实际渲染检查已靠DOM条件判定，但尚无视觉截图审阅；增加真实WebKit截图或能支持的浏览器视觉检查。不能把DOM存在等同于公式／UI视觉完全通过。
6. 补充故障恢复、持久化重启／渲染进程恢复验证。当前样例没有真实模型、SQLite或完整笔记库，不可称完整产品稳定。
7. 按约500MB预算（明确PSS与RSS、共享页、GPU口径）比较，结合开发／交付维护成本，提出有证据的技术选择，不预定Rust获胜。
8. 更新正式需求／设计／选型结论，提交同步GitHub。最后清理未采用方案产物与新依赖，保留精简复现材料和结果。

## 磁盘与清理

暂停时验证目录约：build1.3GiB、cargo-cache253MiB、web/vendor9MiB（后续以du复查为准）。root文件系统当时可用148GiB；用户仍明确要求节省空间。

依赖安装事务是DNF **23**，完整记录保存在 `results/dependency-transaction-23.txt`。该事务除了新增包还包含GLib、AT-SPI和systemd等版本替换／降级。**不要为清理盲目执行整笔history undo**；只针对本次未采用的新增包核对依赖后卸载，不擅自调整已有系统服务。sudo需要用户密码；用户已自行完成安装，无需索要密码。清理系统包若不能自动执行，准备准确、最小的命令让用户在本机运行。

本续接记录先保存本地；尚未为本轮测试创建新Git提交／同步结果。原仓库最新提交仍是3bb543b。
