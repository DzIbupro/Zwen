# Zwen 项目交接文档

更新时间：2026-10-31，时区 Asia/Shanghai。面向没有本项目聊天记录和记忆的 AI。

> 历史基线：本文记录 2026-10-31 交接时的状态。之后完成的“截图附加胶囊草稿”本地单元见 `docs/development-progress.md` 与 `docs/validation/2026-10-31-screenshot.md`；本文不再逐项更新。

**本文可以单独发送给接手 AI。它包含需求、现状和工作边界；具体实现仍以可读取的仓库代码为准。不要把“计划采用”“原型验证”“合成测试”理解为产品已实现。**

## 1. 先读这一页

Zwen 是 Fedora + Niri / Wayland 上的个人桌面学习助手。用户要一个好看、稳定、正常工作内存约500MB或接近500MB的工具：完整问答使用大窗口，阅读中的解释、翻译和截图提问使用小胶囊；以后把用户保存的材料交给接入的AI自动分类、关联，形成可视化知识网络。

正式技术路线已经确定并开始实现：**Rust + Tauri 2 + Svelte/TypeScript，SQLite保存应用状态，后续Markdown保存笔记正文**。不要因看到旧Python文件就改用Python重做，也不要无证据重新比较Go/Java/Rust。

当前已经实现文字问答、历史与恢复、双窗口模式、显式选区导入。**下一开发单元是截图附件，随后接视觉模型；知识库、AI整理和图谱还没有实现。**

用户本机曾成功调用一次智谱文字模型，但真实多轮、停止、图片和长期稳定性不能据此勾选为通过。

## 2. 仓库、版本和交接状态

| 项目 | 信息 |
| --- | --- |
| 本机工作目录 | `/home/dzibupro/Zwen/Zwen`，注意是两层Zwen |
| 私有GitHub仓库 | https://github.com/DzIbupro/Zwen |
| 当前开发分支 | `feat/desktop-foundation` |
| 已有草稿PR | https://github.com/DzIbupro/Zwen/pull/1 ，尚未合并 |
| 本次交接核对的HEAD | `9a3efc9f059ca51fd571d7a087ce2c7659459800` |
| 最近功能提交 | `b1ee2a8b21ba155c0b8ad146c3f8aaa6acb2e983`：显式选区导入 |
| 前一文字问答功能提交 | `dfd63ac12a01174449d4c35fc7750a0d5ddab621` |
| 普通程序 | `artifacts/zwen`，已构建，不含`desktop-smoke`测试特性 |
| 程序校验 | `artifacts/validation/ordinary-binary.sha256`，交接时核对通过 |
| 当前进度状态 | `docs/development-progress.md`中的`unit-complete`，不是额度暂停 |

写本文前工作区干净；本文是本轮新增加的交接文件，是否已提交请再查`git status`。不要为了取得“干净状态”删除它或用户修改。

旧仓库/目录名是study-assistant，显示名称已统一为Zwen。旧标识仅在数据兼容处有意保留。`origin/main`等本地跟踪引用未必实时，涉及合并前重新核对远端。保持现有草稿PR，不自动合并、强推或重置分支。

其他机器需要先获得该私有仓库的读取权限。只有本文而没有代码时，不能假装已经查看代码或运行测试。本机路径、缓存、二进制和验证产物不会自动随Git克隆过去。

## 3. 用户已经确认的产品要求

### 3.1 环境、资源和稳定性

- 主要环境：Fedora 45、Niri 26.04 / Wayland，约16GB内存；实际环境会变化，必要时重新确认。两块屏幕，曾验证100%与150%缩放。
- 常态内存约500MB以内或接近500MB可以接受；避免1GB、1.5GB、2GB级常态占用。旧100/180MiB预算已经废弃。
- 优先级：数据可靠和稳定 → 合理资源占用 → 核心功能与视觉交互 → 开发便利。不能为了省内存去掉表格和公式。
- 统计整个应用进程树，分别报告PSS、RSS与GPU统计边界；不能只报Rust主进程，也不能拿PSS充当RSS。
- 不能承诺绝对不崩溃。已保存数据应能恢复；网络、数据库、渲染等可恢复失败应有提示和重试路径，不能异常退出或静默丢内容。
- 存储空间有限：复用选中架构的依赖、锁文件与缓存，及时清理本项目废弃编译产物，不做全局autoremove或删除未知目录。

### 3.2 两种界面

- 普通打开、完整问答、历史和后续知识管理：大窗口。
- 选区解释、翻译、代码解释、截图附加草稿：小胶囊，不能直接弹出大聊天界面。
- 当前工程默认胶囊宽约480、初始高96逻辑像素，顶部居中避开顶栏；回答向下增长，达到屏幕高度约55%后内部滚动。具体数值是工程初值，不是用户不可改变的硬约束。
- 宽度固定；宽表格、代码和公式在自身区域横向滚动。输入和操作区仍可用。
- 胶囊与大窗口共享草稿、回复和生成状态；展开不能重发问题或丢状态。用户最后一次收起/展开操作优先于迟到的读取结果。
- 正常显示Markdown段落、列表、引用、代码、表格、行内与块级公式。用户不要论文出版或复杂科研排版系统。
- Vida仅作为桌面UI交互参考，不复制品牌与资产，也不引入它的主动推荐、自动操作电脑等功能。

### 3.3 材料和模型边界

- 快捷唤出只打开助手，**不自动读取或附加当前应用内容**，也不把窗口标题自动当文献来源。
- 材料来自用户手动输入、显式选区、显式复制或截图；只有用户明确发送才交给模型。
- 截图先附加胶囊草稿，显示缩略图，允许补问题、删除、重截和取消；不自动发送。
- 支持流式回答、停止、追问和历史。本机默认远程推理，不常驻大模型、OCR或向量服务。
- 自动划词后弹浮条属于后续目标，当前是“选区＋显式快捷入口”，二者不要混淆。

### 3.4 最终知识整理目标

- 用户选择保存/纳入的问答、摘录、笔记和截图统一管理，Markdown文件是笔记正文主要存储形式。
- 用户接入的AI自动识别知识点、主题和栏目，支持一个知识点属于多个主题，识别跨栏目关联，形成可视化网络。
- 初次启用整理时明确模型和材料范围；范围内可以增量自动整理，不要求逐条批准关系，也不能默认上传所有历史聊天。
- 支持人工编辑、合并、拆分、改名和重新归类；保护人工修改，AI变更可追溯、可撤销。
- 关系连接证据材料；模型推断独立标记。材料修改/删除后，依赖它的关系需要失效或重算，不留下悬空证据。
- 可视化包含节点/连线、缩放拖动、搜索、主题筛选、按需展开；点节点看笔记，点关系看说明和证据。不能一次加载整个大库。

## 4. 当前真实完成度

| 功能 | 当前状态与边界 |
| --- | --- |
| Rust/Tauri/Svelte正式工程、单实例 | 已实现；Python和技术对照样例是历史材料 |
| 完整窗口、固定宽胶囊、动态高度/滚动 | 已实现；同一WebView和业务状态，Niri调整只针对本应用窗口 |
| Markdown、表格、公式、代码 | 已实现；依赖本地打包，内容净化 |
| 草稿自动保存、关闭保护、重载恢复 | 已实现；约500ms防抖，发送前保存；整个进程重启只保证最近成功落盘状态 |
| 非敏感模型设置、环境密钥 | 已实现；密钥不存SQLite、不通过IPC返回、不写日志 |
| Rust HTTP/SSE文字流式、停止、多轮、历史 | 已实现；有界输入/输出，错误保留已收到回答，不自动重试 |
| 磁盘故障时回答保护 | 已实现；未保存回答留Native RAM，阻止覆盖和未保存退出；不等于已持久化 |
| 智谱真实文字调用 | 用户本机成功过一次；不是完整真实模型验收 |
| 显式选区导入 | 已实现三个CLI入口及胶囊导入；真实浏览器/阅读器/终端矩阵仍待验收 |
| 全局快捷键安装 | 尚未自动配置，需Niri绑定显式入口；不要宣称快捷键已经装好 |
| 自动划词浮条 | 未实现 |
| 截图附加草稿、缩略图、持久化、发送 | 未实现；只有早期系统接口原型证据 |
| 图片请求、视觉模型路由 | 未实现 |
| Markdown知识库、收藏搜索、AI整理、可视化图谱 | 未实现；技术样例里的图不是正式产品功能 |
| 外链通过系统浏览器打开 | 尚未接入，按当前README边界处理 |
| 30分钟/8小时、100次正式产品唤出等验收 | 尚未完成；旧样例的短测不替代正式产品验收 |

## 5. 代码地图和不能破坏的约定

以下路径均相对于仓库根目录；不需要从旧聊天恢复源码。

| 位置 | 职责 |
| --- | --- |
| `crates/core/src/lib.rs` | 私有SQLite、草稿、错误、256KiB草稿上限 |
| `crates/core/src/provider.rs` | 服务配置验证、规范化、环境密钥包装、配置CAS |
| `crates/core/src/stream.rs`、`completion.rs` | 有界SSE帧和聊天完成协议 |
| `crates/core/src/journal.rs` | 问答记录、检查点、历史、上下文和会话配置绑定 |
| `crates/core/src/cli.rs`、`selection.rs` | 启动动作、PRIMARY有界读取、单槽选区收件箱 |
| `crates/core/src/surface.rs`、`paths.rs` | 窗口边界/generation、旧数据目录兼容选择 |
| `crates/transport/src/` | reqwest HTTP/SSE、超时、取消、错误码、资源边界 |
| `src-tauri/src/main.rs` | 单实例、启动路由、IPC注册、数据目录、退出保护 |
| `src-tauri/src/chat.rs`、`provider.rs`、`selection.rs` | 原生业务通路，重任务离开GTK线程 |
| `src-tauri/src/windows.rs`、`niri.rs` | 窗口意图/几何、Niri适配 |
| `src-tauri/build.rs`、`capabilities/main.json` | 注册命令和最小权限；新增IPC必须同步修改 |
| `frontend/App.svelte`、`Capsule.svelte` | 主交互、双模式展示；不另开一个重复请求的独立聊天状态 |
| `frontend/lib/` | draft/chat/provider/history/selection/surface控制器、桥接和渲染 |
| `frontend/SelectionNotice.svelte` | 待处理选区、重试保存、人工检查和丢弃 |
| `crates/*/tests/`、`frontend/tests/` | 单元/HTTP/控制器测试 |
| `src-tauri/src/smoke.rs`、`src-tauri/tests/` | 仅desktop-smoke特性编译的原生合成验证 |
| `scripts/validate_*.py` | 临时库、合成服务、自有窗口的测试驱动 |

重要约定：

1. 草稿/配置用revision CAS，不覆盖其他写入；所有数据库I/O离开UI线程，损坏或未知schema不自动删库重建。
2. 聊天先保存问题再联网；首片段、过程检查点、终态保存。发送接受后仅清空没有继续编辑的草稿。
3. 会话绑定规范化服务根地址、模型、密钥变量名；配置改变时不把旧上下文转发到新服务，要求新会话。
4. 最近至多8轮上下文、合计正文256KiB；历史每页20条摘要，正文按需读取；超限必须明确提示。
5. HTTP禁自动重试、重定向和隐式代理；连接/单次读取/总超时10/30/120秒。输出256KiB、线路8MiB；SSE块64KiB、行/事件256KiB。
6. 只将明确完成标记和合法结束协议判为成功；断流不能冒充完整回答；停止pending也必须使迟到start失效。
7. 未保存终态回答保留Native RAM快照，重建渲染器仍可见；新发送和退出不得丢弃它。
8. 内容不能执行shell或任意文件操作；保持CSP、Markdown净化和最小IPC权限，不开放通用命令执行接口。
9. Wayland不要假设set_position有效；CSS像素、devicePixelRatio、Tauri scaleFactor和逻辑像素不能混用。Linux不要简单设resizable=false：曾出现上游200像素最小高度问题，当前用min/max锁定尺寸。
10. 窗口操作有独立意图token并共用操作锁；旧尺寸generation和迟到捕获不能盖过用户后来的窗口操作。

## 6. 选区链路的特殊注意事项

入口是`select`、`explain-selection`、`translate-selection`，后两者添加可编辑的解释/翻译成中文前缀。

- 只调用`wl-paste --primary --no-newline --type text`；不回读CLIPBOARD，不模拟Ctrl+C，不监听鼠标。
- 读取64KiB上限、750ms目标超时，非阻塞管道；拒绝超限、空白、NUL、非法UTF-8和非零退出，不输出helper stderr或正文。调度/回收不承诺硬实时。
- 先读完再请求聚焦；重复快捷调用不能抢在第一次读取完成前激活窗口。
- 原生单槽RAM收件箱不能覆盖未确认材料；前端先附加现有草稿并保存，再确认。
- 渲染器在确认前重载：只显示待检查材料，不自动再加一遍；同进程重试保存不得重复附加。进程强制退出不保证恢复尚未入稿的RAM材料。
- 手动导入、丢弃和状态刷新必须互斥，迟到的成功/失败刷新都不能提前清除busy。这里曾复现“已丢弃材料被再次添加”的竞态，已有回归测试。
- 未处理材料阻止正常退出；读取完成不能把已经收起的窗口重新弹出，也不能把主动展开的大窗口缩回胶囊。
- PRIMARY只反映其所有者提供的内容，不能凭此证明每个应用刚刚选中了什么。当前合成wl-paste验证不等于跨应用通过。

## 7. 数据和凭据

默认新目录：`$XDG_DATA_HOME/io.github.dzibupro.zwen`，通常为`~/.local/share/io.github.dzibupro.zwen`。

**本机为了兼容旧草稿，实际已使用过`/home/dzibupro/.local/share/io.github.dzibupro.studyassistant`。** 由`paths.rs`选择；不要为了统一名字直接迁移、删除或重建旧数据。

- `ZWEN_DATA_DIR`可显式覆盖，主要用于测试；另有旧环境变量兼容，见main.rs。
- `state.sqlite3`草稿schema1；`provider.sqlite3`配置schema1；`chat.sqlite3`聊天schema2。不要将聊天schema2误当草稿迁移版本。三库独立。
- Linux新建目录0700，数据库与sidecar0600。SQLite有长度限制，超大/损坏文件不能触发无界载入或自动修复。
- 不提交数据库、WAL/SHM、草稿、图片、密钥、原始机器日志；`artifacts/`等为git-ignored本机产物。

用户选择的服务：

```text
API根地址：https://open.bigmodel.cn/api/paas/v4
文字模型：glm-4.7-flash
未来视觉模型：glm-4.6v-flash
密钥环境变量名：ZWEN_API_KEY
```

当前密钥由用户在本机`./scripts/run-with-key.sh`中不回显输入，只进入应用环境，不写.env。**禁止读取`/proc/.../environ`或其它进程环境找Key，也不要要求用户把Key发到聊天。** Secret Service配置界面尚未实现，接手后不能假定已存在。

用户首次测试HTTP429，重启诊断新版后手动重试成功：只读元数据确认complete、无错误、收到3字符；没有读取问题或回答正文。第一次429的具体原因未知。诊断仅有界读取错误体并提取数字业务码，不显示完整服务message，不自动重试。以后出现新码要查智谱官方文档，不凭429推断欠费，也不自动反复请求。

## 8. 本机运行、构建与测试

### 8.1 普通程序

在仓库根目录运行：

```bash
./artifacts/zwen show
./artifacts/zwen capsule
./artifacts/zwen recover
./artifacts/zwen quit
./scripts/run-with-key.sh
```

上述是不同入口示例，不要为了验证文档一次全执行。选区命令应由Niri快捷键从原阅读应用触发，不要先切到终端导致选区丢失：

```text
/home/dzibupro/Zwen/Zwen/artifacts/zwen select
/home/dzibupro/Zwen/Zwen/artifacts/zwen explain-selection
/home/dzibupro/Zwen/Zwen/artifacts/zwen translate-selection
```

尚未自动安装这些全局绑定。更改用户Niri配置前先检查已有绑定和授权，不盲目覆盖。

### 8.2 工具和缓存

本机曾验证Rust/Cargo1.98.1、Node24.22.1、pnpm11.25.0；版本以当前工具输出和锁文件为准。

Node/pnpm若不在PATH，当前机器曾使用：

```text
Node目录：/home/dzibupro/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin
pnpm程序：/home/dzibupro/.cache/codex-runtimes/codex-primary-runtime/dependencies/bin/fallback/pnpm
```

这些是本机路径，不要求别的AI环境也存在。优先使用已安装工具，不要直接重新装一套。

`CARGO_HOME="$PWD/.cache/cargo"`是本项目选中依赖缓存；`node_modules`也保留。`target/`上轮已清理约1.61GiB，不存在不代表代码丢失。普通程序和校验文件保留。不要恢复已放弃的GTK4/WebKit6对照开发包；Tauri在Linux使用WebKitGTK4.1。Node只用于开发构建，不随普通程序常驻。

### 8.3 需要修改代码时的检查

```bash
cd /home/dzibupro/Zwen/Zwen
pnpm test
pnpm check
pnpm build
CARGO_HOME="$PWD/.cache/cargo" cargo test --offline
CARGO_HOME="$PWD/.cache/cargo" cargo build --offline --release -p zwen-desktop -j 2
```

Cargo默认测试core和transport，不编译GUI；HTTP测试只绑定回环合成服务，沙箱需要允许本机监听。离线缓存不全时先诊断缺项，再有选择联网补齐；不要把环境失败误判成业务失败。不要在尚未构建dist时直接编译发布壳。

若依赖确实缺失，前端按锁文件安装：`pnpm install --frozen-lockfile --store-dir .cache/pnpm-store`。不要每轮无条件重装。

原生合成测试另用测试构建：

```bash
CARGO_HOME="$PWD/.cache/cargo" cargo build --offline --release -p zwen-desktop --features desktop-smoke -j 2
python3 scripts/validate_selection.py "$PWD/target/release/zwen"
python3 scripts/validate_desktop.py "$PWD/target/release/zwen"
python3 scripts/validate_capsule.py "$PWD/target/release/zwen"
python3 scripts/validate_provider.py "$PWD/target/release/zwen"
python3 scripts/validate_chat.py "$PWD/target/release/zwen"
python3 scripts/validate_chat_fault.py "$PWD/target/release/zwen"
```

按变更选择必要回归，不是每次都盲目全跑。**运行前确保用户已保存退出真实Zwen**，否则单实例可能路由到用户进程。不能擅自kill用户实例。脚本仅使用临时库、合成文本/密钥、回环服务和自有窗口；磁盘故障注入绝不能指向用户目录。

交付前重新普通构建去掉desktop-smoke，再将产物原子替换到`artifacts/zwen`、记录SHA256。不要把测试二进制给用户日常使用。没有代码变更时不必为读文档重建整个工程。

浏览器`pnpm dev`只预览127.0.0.1:1420，草稿使用localStorage；不是桌面SQLite、模型连接或原生窗口通过的证据。开发壳入口见`scripts/dev-desktop.sh`。

## 9. 已有测试证据及其限度

- 最近完整Rust core/transport：60项通过；前端：45项通过；Svelte0错误/0警告；普通构建通过。这是上一开发单元的结果，修改代码后应按影响重新验证，不把它当永远有效的实时状态。
- 选区原生合成3场景通过，覆盖冷/热单实例、失败保留、重复触发、未确认退出、重载人工检查、CLI recover、读取中隐藏保持未映射及展开保持大窗口；基础草稿3场景和胶囊几何回归通过。
- 文字问答合成覆盖流式、表格公式、停止、历史继续、配置绑定、渲染器重载/进程重启和磁盘故障保留RAM回答。
- 上轮20轮、每轮90组公式/表格的短时合成负载：进程树PSS峰344.11MiB、RSS586.30MiB；不包含完整GPU核算。不是完整成品已经低于500MB，也不是新选区/未来截图的资源实测。
- 真实跨应用PRIMARY、中文输入法完整矩阵、多屏长期使用、真实模型多轮/停止/图片、100次正式产品唤出、30分钟/8小时仍待验收。

## 10. 下一步：截图附件，先把边界做对

这是下一阶段工作建议，**下面不是现成实现，也不是已经测试通过的API方案**。

已知系统原型证据来自2026-10-29：

1. 当前桌面的Screenshot Portal interactive调用返回response=2，服务提示缺少InteractiveScreenshot；不能未经重测就把Portal作为可用默认方案。
2. Niri截图指定唯一输出路径曾成功，收到了带相同路径的ScreenshotCaptured事件。
3. screenshot命令约62–69ms即返回，**退出成功不代表图片已保存**。
4. 用户曾确认按Esc取消，独立新路径60秒观察无文件；这只证明当次没有新图，不证明程序已能实时识别取消。

建议接手顺序：

- 先重新核对本机Niri截图接口及当前能力，选定最小适配方式；不要直接复用上一张图片。
- 每次捕获独立request ID和私有输出位置，明确成功、取消、过期、迟到结果的归属；旧结果不得附加到新草稿，也不得覆盖用户最新窗口意图。
- 先完成有界图片验证和本地草稿附件：限制压缩文件、解码像素、数量和缓存，防止图片让内存从约500MB跃升到1–2GB。
- 接入胶囊缩略图、补问题、删除/重截/取消和持久化；任何上传都等用户明确发送。
- 再实现视觉模型配置/路由与图片请求，核对智谱官方协议，不能把图片发给当前文字模型，也不能拿“只把文件名放进问题”冒充图片接入。
- 设计图片与会话/历史/追问的关联，明确切模型后的上下文边界；有必要迁移时先设计兼容、备份和测试，不强改三库现有schema。
- 合成服务/临时图片验证失败路径后，再在用户控制的凭据和明确选择的材料上做一次真实验证；没有Key不构成读取进程环境的理由。

本轮没有选定图片库、附件schema、截图取消检测算法或视觉请求实现。接手AI应根据代码和本机证据设计，不要把自行推测伪装成之前已经达成的决定。

## 11. 随后阶段

1. Markdown知识库：收藏/纳入、统一目录、正文/附件、来源、搜索、人工编辑与导出。
2. AI自动整理：主题/栏目、多归属、关系证据、增量任务、版本失效、撤销与人工保护。
3. 可视化知识网络：先局部、按需展开、搜索筛选和证据回看，做真实规模性能验收。
4. 自动划词浮条：另验鼠标释放、PRIMARY、弹窗位置和跨应用支持；不要干扰显式快捷入口。
5. 持续资源/稳定性、中文输入法、多屏、打包安装和发行验收。不要无依据给项目加新常驻服务。

图谱展示在技术决策中拟用Cytoscape，旧对照样例用过3.34.4；它尚未接入正式前端，实际依赖仍以package.json和锁文件为准。

## 12. 接手AI的执行约定

- 用户习惯授权后直接推进；不要重复问已确认的产品范围、语言或同一项权限。每次先核对当前代码和进度，不重做已完成单元。
- 有新需求或真正影响产品的缺项再问；能从仓库知道的信息先查仓库。实施选择要与用户指定需求区分。
- 每次一个可验证最小单元，必要边界测试先失败后修复；有意义地报告改了什么、证据和未验证项。不为文档小修改重跑全部GUI测试。
- 用户要求五小时额度接近耗尽（约95%已用）时完成手上最小任务、保存代码/测试/下一步/重置时间再暂停；若有账户工具，每个开发单元后核对300分钟窗口。
- 历史进度中的32%、重置时间等是当时快照，不是实时用量。新的AI若没有用量工具就坦白，不能编造。
- `unit-complete`表示上一单元完成；`quota-paused`才表示因额度暂停。用户直接要求继续开发时可以开始下一单元并标记in-progress；旧额度自动续接规则不能让接手AI永久卡在unit-complete。
- 自动后台续接须防止与正在进行的人类回合重复启动；额度未恢复/状态不变时保持安静，只通知成果或需用户处理的阻碍。
- 不继承“此前已经打开/登录某个页面”的假设。GitHub连接器、CLI凭据、浏览器控制和原生操作能力要重新检查；权限不足用安全替代，不读取隐藏凭据。
- 当前本地工具曾禁用原生鼠标键盘控制，只允许浏览器控制和自有应用的测试；另一AI的能力可能不同。不能声称执行了实际没有工具完成的拖选、Esc或截图操作。
- 开发推送沿用私有分支/草稿PR，不自动合并主分支；遵循接手环境的技能、仓库说明和用户权限。
- 本次用户只要求制作交接文档，不等于要求在制作文档时继续实现截图。

## 13. 进一步阅读

这些文件都在仓库中；本文摘要与它们有差异时，先核对当前代码和用户最新要求，旧日期计划不能覆盖更新后的实现。

1. `docs/requirements.md`：产品范围和用户原意。
2. `docs/design.md`：总体架构、阶段与稳定性边界。
3. `docs/development-progress.md`：最近状态和下一任务，需结合git log。
4. `docs/decisions/2026-10-29-technology-selection.md`：为何选择Rust/Tauri，以及资源测量限制。
5. `docs/validation/2026-10-30-selection.md`与对应`docs/superpowers/specs/2026-10-30-selection-design.md`、`docs/superpowers/plans/2026-10-30-selection.md`：选区实现、评审及回归。
6. `docs/validation/2026-10-30-provider-chat.md`、`docs/superpowers/plans/2026-10-30-provider.md`、`docs/providers/zhipu.md`：问答与智谱。
7. `docs/validation/2026-10-29-fedora-niri.md`：早期系统接口原型，特别是截图成功/取消边界。
8. `CONTRIBUTING.md`与`README.md`：构建、运行、测试和当前功能说明。

## 14. 可复制给接手AI的启动指令

> 请接手Zwen项目。先完整阅读我提供的这份AI-HANDOFF.md，再读取仓库的docs/requirements.md、docs/development-progress.md，检查git status和最近提交，确认与交接基线的差异。不要重选技术栈、重做已完成的文字问答或选区单元，也不要把历史合成验证当成真实环境通过。若你有本机仓库权限，请按既定需求直接推进下一最小单元“截图附加胶囊草稿”，先核对Niri接口、取消/过期/迟到归属和图片资源边界，再接视觉模型；不自动读取当前应用或上传材料。需要凭据时只让用户在本机输入，不读取其他进程环境，不让用户把Key发到聊天。遵循约95%额度时保存并暂停的约定。若缺少仓库或工具权限，明确指出具体缺项，不编造已实施或已测试。
