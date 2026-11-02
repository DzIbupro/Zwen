# 2026-11-03 独立操作条可行性实验续接

状态：**quota-paused**。用户已授权继续到只剩很少额度；本轮从周97%推进到99%（最后核对300分钟92%），已完成最小合成实验并保存。未使用重置卡。用户可重置后说“继续”，届时重新核对账户，不只等待原11月4日时钟。

成果：独立layer-shell操作条319×44、NONE键盘模式、无exclusive zone、焦点保持；AT-SPI可在单独启用桥的自有测试窗口提供局部选区坐标，但Niri26.04平铺窗口缺绝对位置；一次性透明层能取得鼠标附近坐标并立即销毁，焦点保持。原型都未接入主程序、不代表精确选区右上角或无输入影响。没有读取用户选中文字/密钥/数据库，没有调用模型或截图。

代码scripts/spikes/{selection_geometry,selection_toolbar,selection_anchor,pointer_anchor}.py；实验与边界见[记录](validation/2026-11-03-independent-toolbar-spike.md)。下一任务优先验证指针探针不会吞点击/拖动及取消时序，再串联独立小操作条→点击命令→激活胶囊，保留明确定位来源与精度。不要再把候选直接弹胶囊当完成。旧普通程序未替换，Zwen仍处于用户已退出状态。

---

# 2026-11-03 划词交互纠正：独立选区操作条（尚未完成）

状态：**quota-paused**。本次账户工具确认300分钟已用81%、周额度已用97%；按用户先前“接近额度上限完成最小任务后暂停、重置后续接”的约定保存当前单元，周额度重置时间北京时间2026-11-04 14:48:19。用户否定候选直接弹胶囊：应在选中词句右上方显示独立小操作条，点击后才打开胶囊。按最新需求停止部署旧胶囊修补版。用户已保存退出旧实例；尚未重新启动。托盘/截图等已部署版本仍保存在artifacts/zwen。

已完成未部署中间修补：全局summary30×30误伤选区菜单的CSS收窄，候选时隐藏旧回答/输入框，问一问准备新会话；新增RED→GREEN组件测试，112前端通过、类型检查0/0、Vite与普通桌面构建通过。浏览器实际共享组件截图artifacts/validation/candidate-menu-fixed.png，只证明中间布局修补，不代表用户所要求独立浮条。已保存为7c045ce检查点，未部署，不能误记为交付。

下一最小任务：依据更新spec先验证AT-SPI选区边界/屏幕坐标与不抢焦点layer-shell窗口；确定来源应用支持后实现独立操作条。Niri26.04现有IPC无cursor查询，Tao Wayland返回(0,0)，不可直接依靠Tauri定位。已核实libgtk-layer-shell.so.0、libatspi.so和Atspi2.0 introspection可用；没有读取用户真实选区内容。

---

# 2026-11-03 系统托盘

状态：**unit-complete**。已加Zwen原生托盘与打开主窗口/胶囊/截图/设置/保存并退出菜单，隐藏后可从托盘召回；退出沿用保存保护。前端111、桌面27（另1忽略）、类型检查与普通构建通过；真实会话D-Bus隔离托盘注册/菜单/退出注销测试通过。详见[验证记录](validation/2026-11-03-tray.md)。普通程序已更新，SHA256：3c8d4ff46d8197cdeea7057863f0a13c4bb7006ddabd439ef6bb29353cc230cb。

后续：用户确认桌面栏实际图标外观与菜单点击体验；不把D-Bus注册检查当成像素/鼠标实测。仍保留既有截图Shift+Alt+S和划词功能。以下为历史记录。

---

# 2026-11-03 划词动作菜单

2026-11-03 快捷键调整：按用户要求将本机 Niri 的 Zwen 截图快捷键从 Ctrl+Alt+A 改为 **Shift+Alt+S**，目标仍为 artifacts/zwen screenshot。配置已备份、niri validate通过，并调用load-config-file重载成功；尚未实际按键触发截图。以下旧快捷键记录属于历史。

状态：**unit-complete**。用户确认的新需求已完成：问一问Zwen/翻译/详细解释/其他、自定义提示词设置与默认文档目录，完整回答后收藏并导出Markdown。详见[验证记录](validation/2026-11-03-selection-actions.md)与[实现计划](superpowers/plans/2026-11-03-selection-actions.md)。自动菜单需启动程序并在模型设置开启，来源应用需提供PRIMARY，不宣称任意页面都可读取。

验证：前端111、核心/传输126、桌面26通过（另1真实钥匙环测试忽略）；actions串联驱动通过，原有selection/chat/knowledge驱动通过。仅临时库/合成选区/本机合成模型，无生产资料或真实服务请求。普通程序SHA256：88791a2dd5f7e84f29d8a33f939e402f529093f3b1ffba1013a2a7a38a22f9f0。普通构建无smoke标记。

两项审查问题已修补并复审：自身选区取消保留原候选；配置版本原子校验后绑定请求。额外修补回归暴露的中文附件临时文件名长度溢出。CUA原生控制仍不可用，真人跨应用划词兼容性未验收。短聊天PSS296.12MiB/RSS536.92MiB不等同于长期资源验收。

下一步：用户启动新版开启自动划词菜单后，验证常用浏览器/终端/文档阅读器真实划词；失败时只收集错误和程序名，不读取选区隐私。后续可补动作输出进程重启后的待保存恢复、大库分页和持久编辑草稿。历史暂停状态不作为自动重启依据。

---

# 2026-11-03 知识库编辑交错测试

状态：**unit-complete**。本次 heartbeat 按约定完成一个最小可验证单元。最后账户核对300分钟已用13%、周额度86%，未达到quota-paused条件，不虚报额度暂停。

修补切换页面丢失未保存笔记、迟到保存重新打开已关闭笔记；保存期间锁定字段，失败保留编辑，正常退出前提示先保存或取消。编辑快照仅在本次运行内存中，不属于崩溃恢复。

验证：前端100项通过，Svelte 0错误/0警告，Vite与普通桌面构建通过；原生隔离知识库驱动v7通过（新增编辑→历史→知识库→保存落盘校验），没有调用真实服务或访问生产库。普通二进制SHA256：8d0f297c1a1d60277f1084b2bcad255e192b7a287e1b39a7e94cbd50cfcd62c4。详情见[本轮验证](validation/2026-11-03-note-editing.md)。CUA原生控制仍不可用，应用内驱动不等同于真人原生鼠标键盘验收。

下一最小任务：检查知识库搜索与非搜索操作的并发、主题/笔记异步打开后的导航取消；随后推进大库分页和持久编辑草稿。避免重做已通过的跨页编辑保留。没有用户新指令或quota-paused恢复时不要重复启动本单元。

---

# 2026-11-02 交互测试续接点

状态：**unit-complete**（本段为上一轮历史）。上一轮暂停记录：当前300分钟窗口最后核对已用93%，按约定完成搜索修补与收尾后暂停；重置时间北京时间2026-11-02 23:54:05，后续仍须账户工具确认恢复，不只看时钟。

本轮CUA原生应用清单为空、listWindows不可用；不能声称完成真人原生鼠标键盘验收。已在独立端口浏览器中实际输入中文、预览表格公式、切换胶囊/完整窗口、运行演示与搜索；截图证据及范围见docs/validation/2026-11-02-interaction.md。未访问用户资料或真实模型。

搜索修补：待完成查询期间的新输入改为合并到最新查询，防止输入丢失；新增回归RED→GREEN，前端96项/Svelte0/0/Vite通过。普通程序已重建并通过哈希校验（95f46951bdc0ffbc52f4085ddcc19a6bad6b63b82f94836986c4df42be2026df），本轮target已清理约1.26GiB；选中依赖缓存保留。

下一最小任务：先核对本轮普通构建/提交状态，再继续知识库快速输入与导航/保存交错、失败恢复的隔离端到端测试；若原生工具恢复则用实际控件，不擅自读取生产库或Key。上一轮Rust118+desktop16、原生9/9属于前一轮审查证据，本轮只有前端代码变化，未重新执行整个原生矩阵。

---

# Zwen 当前进度（2026-11-02）

状态：**unit-complete**。本轮依据用户授权完成独立审查、优先级缺陷修补与用户logo接入。审查基线6bb9e7d；详情见[审查结果](AUDIT-RESULT-2026-11-02.md)。Rust core/transport118、desktop16（另1个真实钥匙环测试ignored）、前端95均通过，Svelte0/0，原生合成9/9。功能提交baa5150已推送私有分支；普通程序已更新，无desktop-smoke，SHA256为1d5529692b4e33f431db5d278aa54af5f21be78546e29049207b6b5266a24727。已清理本轮target约1.64GiB，保留程序和选中依赖缓存。未对真实模型、真实跨应用/多屏或长期稳定性作新的通过宣称。下文保留之前轮次历史，若冲突以本段、最新审查报告和代码为准。

用户logo：assets/brand/zwen-logo-original.png保留原图；前端96px、Tauri256px资源已接入。实测普通窗口app_id=zwen，用户级zwen.desktop和图标已更新；没有改动Niri快捷键。

下一步优先真实环境/资源验收和大库后端分页；不要重复实现已完成单元。额度接近95%时按原约定保存暂停，不能把历史百分比当实时值。

---

# Zwen 当前进度（2026-11-01）

状态：**unit-complete**。2026-10-31 完成截图附件与视觉模型路由两个单元；2026-11-01 依据用户真实使用反馈完成”截图期间隐藏助手窗口”修复单元。**用户本机报告真实截图提问已可用并收到完整回答；原生合成回归因用户实例运行中尚未重跑（见验证节）。** 本环境没有账户/用量工具，未核对 300 分钟窗口，不编造百分比或重置时间。

## 源码、入口

- `/home/dzibupro/Zwen/Zwen`；分支 `feat/desktop-foundation`，GitHub https://github.com/DzIbupro/Zwen ，草稿 PR1。
- 交接基线 `9a3efc9f059ca51fd571d7a087ce2c7659459800`（与交接文档一致）；本轮本地提交依次为 `ea4ff0e`（交接文档）、`d1138db`（截图单元）、`40f6479`/`8da1453`（证据与哈希）、`2027846`（视觉模型路由）。
- **推送状态：2026-10-31 已推送 `9a3efc9..4d9e5a1` 到远端 `feat/desktop-foundation`，草稿 PR1（OPEN）同步更新。** 推送经由用户在本机浏览器完成 GitHub CLI 设备流授权；`gh` 已以用户级方式安装到 `~/.local/bin`，令牌仅存于本机 `~/.config/gh`，未进入仓库或聊天。
- 普通程序 `artifacts/zwen`，不含 `desktop-smoke`；SHA256 `fb4fcaa2391ba16f4cebfd96825bd751def3cd23f0f1f4ae9a0fe64071e7af27`（2026-11-01 失败保留重试材料版），同时记录在 git-ignored `artifacts/validation/ordinary-binary.sha256`。
- 新入口：`./artifacts/zwen screenshot`（兼容 `capture-screenshot`），胶囊也有截图按钮。**2026-11-01 用户确认后已在其 Niri 配置绑定 `Ctrl+Alt+A` → `spawn zwen screenshot`**（niri validate 通过，配置热重载生效；Niri 绑定不区分左右 Alt）。选区三个入口仍未绑定全局快捷键。
- `scripts/run-with-key.sh` 规则不变：用户本机不回显输入，只进入应用环境；不读 `/proc` 环境，不要求把 Key 发到聊天。

## 已完成

1. Zwen 统一名称与旧草稿兼容；一个 WebView，完整窗口/固定宽度胶囊、渐长与上限滚动、Niri 浮动定位。
2. 模型设置：规范化 HTTPS/回环 HTTP 根地址、模型与密钥环境变量名；独立 `provider.sqlite3`，revision/CAS，损坏不覆盖。密钥不经 IPC。
3. Rust HTTP+SSE 文本流式，禁自动重试/重定向/隐式代理，超时与取消，停止 pending 可阻止迟到 start。只有明确发送才上传草稿与本会话上下文。
4. 对话问题先存储再联网，首片段/过程/终态保存；`chat.sqlite3` schema2，不迁移草稿 `state.sqlite3` schema1。渲染器重载恢复 NativeRAM/磁盘快照，进程重启只恢复成功检查点并标记中断。
5. 多轮：至多最近 8 轮、总正文 256KiB；配置绑定会话，模型/根地址/Key 变量名变化则拒绝附带旧上下文。历史 20 条分页摘要。
6. 评审修复：未保存回答不能被新发送清空；读取失败可重试；磁盘故障时未保存 NativeRAM 仍能显示。
7. 显式选区导入：`select` / `explain-selection` / `translate-selection`；PRIMARY 64KiB/750ms、单槽材料、只追加草稿不发送；冷/热路由先读后聚焦，重载人工检查、保存/确认失败保留、未确认退出保护，迟到读取不覆盖最新窗口意图。尚未安装全局快捷键。
8. **截图附加草稿（本轮）**：
   - `screenshot` 入口先显示胶囊，再打开 Niri 交互截图；每次请求生成 32 位十六进制 ID 与唯一绝对路径，先订阅 `niri msg event-stream`，再执行 `niri msg action screenshot --show-pointer false --path <path>`。
   - 成功判定为命令退出成功 + 唯一文件出现且大小稳定 250ms + PNG 解码校验通过；命令退出成功不算保存成功。
   - 取消（胶囊按钮或再次触发入口）和 60 秒超时都会丢弃当前路径；迟到文件只清理，绝不附加到新草稿。Niri 的 Esc 取消没有可靠事件，不能实时识别。
   - 新请求开始前清理 `incoming` 中全部 32-hex 请求文件；这些文件在进程重启后不可能再被确认。
   - PNG 单张≤24MiB、边长≤8192 且≤10MP；最多 3 张、合计≤48MiB；缩略图最长边 480；原图/缩略图 0600、目录 0700。独立 `attachment.sqlite3` schema1，不改动三个既有库。
   - 重截在同一事务内替换旧记录，失败保留旧附件；删除只删除指定 ID 的原图/缩略图。
   - 视觉模型未接入：存在截图附件时发送按钮禁用，Ctrl+Enter 也会被拦截并提示，避免把图片或文件名冒充成文字发给当前模型。
   - 窗口意图：请求开始和结束共用一个 intent；用户后来的收起/展开优先，迟到结果不能覆盖。胶囊在截图期间保持可见，若把胶囊区域纳入截图会出现在图片中（当前取舍，待真实验收）。
   - 未上传：图片只落本机数据目录，不进入聊天、日志或模型请求。
9. **视觉模型路由（本轮）**：
   - 模型设置新增可选"图片模型"；留空时序列化配置与旧格式逐字节一致，旧对话绑定不变；填写后按既有机制要求新会话。
   - 发送带截图时路由到图片模型；未配置则前端禁用发送、后端 `image_model_missing` 双重拒绝，不把图片或文件名发给文字模型。
   - 请求按官方 GLM-4.6V-Flash 协议构造 content 数组（`image_url` 裸 base64 + `text`）；负载有界：单张≤4MiB、≤6000px，必要时等比缩小，最小 256px 仍超限则拒绝；transport 独立校验图片数量（≤8）与 base64 合法性，非法负载在任何网络请求前拒绝。
   - 发送接受后附件即消费（与草稿同语义）；流式失败不回收附件，重试保存只保存文字。追问同会话纯文本，历史上下文不回带图片；图片轮 `turn.model` 记录图片模型。
   - 官方协议已核对（`docs.bigmodel.cn` GLM-4.6V-Flash 页：content 数组、裸 base64、多图、stream）；页面无数值限制，4MiB/6000px 为按旧版 GLM-4V 规范的保守假设。
10. **截图期间隐藏助手窗口（2026-11-01，用户真实反馈驱动）**：
   - 用户真实使用报告：截图提问端到端可用并收到完整回答；但选区期间 Zwen 窗口保持可见，页面出现错乱与半透明区域，遮挡待截图区域。此前验证文档记录的"胶囊保持可见"取舍被真实使用否决。
   - 修复：`windows::hide_if_current(intent)` 在发起交互截图前隐藏本应用窗口；结束（成功/取消/超时/失败）后经同一 intent 由 `activate_if_current` 重新显示胶囊。用户期间的收起/展开/恢复操作仍优先，迟到结果不能覆盖。
   - 等待期间的取消路径相应变化：胶囊已隐藏，取消=再次触发 `screenshot` 入口（自动取消并显示胶囊），或等 Niri Esc 后约 60 秒超时；界面内"取消等待"按钮仅在窗口可见时有意义。
   - 修复后的原生合成回归因用户实例运行中未重跑（单实例会路由到用户进程，约定不杀用户实例）；Rust 单元测试 79 项通过，普通二进制已重建，哈希见 `ordinary-binary.sha256`。
   - **真实环境确认**：修复由用户在 2026-11-01 之后的真实使用中验收（选区期间无遮挡即视为通过）。
11. **胶囊过渡残影修复（2026-11-01 晚，用户截图驱动）**：
   - 用户提供选区期间的截图：屏幕中央出现 480×96 的窗口，内容是完整窗口标题栏残影（"快捷胶囊/−/···"）与半透明胶囊内容混合并闪烁。该截图同时证明用户实例仍运行旧二进制（旧行为在选区前打开胶囊）。
   - 根因：`activate_if_current` 先 `show` 再发 `surface-changed`，页面在窗口可见后才切换布局；WebKitGTK 透明窗口在重映射瞬间把旧帧（完整窗口顶部条）画进胶囊帧，且 body 过渡态半透明。
   - 修复：`surface-changed` 改为**在 `show` 之前发出**并等待 120ms 让前端切到目标布局；show 前重查 intent，期间的用户操作仍优先。冷启动 `screenshot` 的 `decorations` 改为与胶囊面一致（`!capsule_surface(action)`）。
   - Rust 单元测试 79 项通过；普通二进制重建，SHA256 `cb053bc475d05d8aebeb5bff820680e06546ca1f1cd7ce917ef3143aa25ac4ad`。原生合成回归仍因用户实例运行未补跑。
12. **模型密钥保存到系统钥匙环（2026-11-01，用户诉求驱动）**：
   - 每次启动都要 `run-with-key.sh` 输密钥成为负担；按设计文档预留的 Secret Service 路径实现：设置页"模型密钥（可选，保存到系统钥匙环）"粘贴一次 → gnome-keyring 条目（属性 `xdg:schema=io.github.dzibupro.zwen`、`zwen-role=model-key`）→ 之后启动自动使用；提供"清除已保存密钥"。
   - 解析顺序：启动环境优先（run-with-key.sh 与合成测试路径完全不变），钥匙环兜底；发送时环境缺失自动读钥匙环。
   - 边界不变：密钥不进 SQLite、不经 IPC 回传、不写日志；界面只显示来源（environment/keyring/未检测到）。
   - 实现细节：zbus 5.19 blocking 客户端（复用既有依赖树，未新增第三方 crate）；plain 会话仅走本用户私有总线；Secret 结构 content_type 按官方规范为字符串（首版误用 au，真实端到端测试抓出后修正）。
   - 验证：**真实 gnome-keyring 端到端通过**（存合成值→读回→删除→确认空，无遗留）；Rust 79+2 项、前端 59 项、svelte-check 0/0；普通二进制 SHA256 `cc2d25f0d881fa7c260f5487f648c46a9b47e0f0258598a6b673df51c9e27256`。详见 `docs/validation/2026-11-01-keyring.md`。
   - 待验证：用户用真实密钥在设置页保存后提问一次（即真实验证）；钥匙环锁定提示；原生 UI 回归待用户退出实例后补跑。
13. **截图等待的重新触发语义（2026-11-01 晚，用户奇偶交替反馈驱动）**：
   - 用户实测规律：Esc 退出 Niri 选区后，第一次按 `Ctrl+Alt+A` 能截图，第二次只唤起胶囊不截图，奇偶交替。根因：Niri 的 Esc 无事件（10-29 已知限制），第一次的等待最长挂 60 秒；期间再次触发被实现为"取消旧等待+仅显示胶囊"，于是 Esc 后的那一按永远不开始新截图。
   - 修复：等待期间再次触发=**重新开始截图**——丢弃旧等待（旧 worker 见 id 变化自行退出并清理自己的请求文件），用新 request ID/路径立即发起新请求；不再只弹胶囊。
   - 残余边界（文档化）：若 Niri 截图界面还开着就再按快捷键，新请求的动作可能被忽略而旧等待已取消，结果是最迟 60 秒后超时提示；建议先完成或 Esc 关闭选区界面再按。
   - Rust 79 项通过；原生 screenshot smoke 新增 restart 阶段（等待中重触发→新请求成功→数量保持 1），因用户实例运行中整套原生回归未跑。普通二进制 SHA256 `d1a1ed2f01c3ccb6f13e12d3f33620e4e272af854d966b6c14336abd999b0b1b`。
14. **失败即保留重试材料（2026-11-01 深夜，用户真实 429/1305 驱动）**：
   - 用户首次真实截图提问被智谱拒绝（429/1305=平台过载，见 `docs/providers/zhipu.md`）；按旧语义"发送接受即清空草稿、消费截图"，一个字都没生成的失败也要重新截图重打问题。
   - 修复：消费时机推迟到**收到第一个回答字**。后端 `run()` 在首个文本增量到达时删除附件并广播 `attachments-changed`；前端 `pendingConsume` 在回答非空时才清草稿（按 requestId 匹配，无 id 时按 prompt 匹配）。429/网络/空回答等零字失败完整保留问题与截图，直接再按发送即重试；开始生成后的停止/失败语义不变。手动删除附件、附件已缺失（AttachmentMissing）等情况按已消费处理，不阻塞流。
   - 失败先行：合成视觉服务先对第一次视觉请求返回真实形状的 HTTP 429+1305，断言草稿保留、附件保留、错误码呈现，再重试成功并消费——`validate_chat_image.py` 全链路断言 3 请求/3 轮（failed→complete→complete）。
   - Rust 79 项、前端 59 项、`svelte-check` 0/0 通过；原生套件因用户实例运行未跑。普通二进制 SHA256 `fb4fcaa2391ba16f4cebfd96825bd751def3cd23f0f1f4ae9a0fe64071e7af27`。
15. **原生验收全套补跑（2026-11-01 深夜，用户配合退出实例）**：
   - 8 项原生合成回归全部通过：desktop、capsule、selection、provider、chat（含内存采样）、chat_fault（磁盘故障注入）、chat_image（含 429 保留重试材料）、screenshot（含等待中重触发 restart 阶段）。完整输出存 git-ignored `artifacts/validation/native-suite.log`。
   - 首轮曾挂 chat 与 chat_image：根因是**测试驱动选择器脆弱**——设置页新增"保存密钥"按钮后，驱动里 `.settings button[type=button]` 先匹配到新按钮，"重新读取"没被点击。产品 UI 无问题；四个驱动（chat/chat-image/stress/provider）改为按文字匹配"重新读取"。教训：驱动选择器应绑定语义文本而非 DOM 顺序。
   - chat 冒烟内存采样（50ms 进程树）：PSS 峰 259.86MiB、RSS 峰 512.02MiB——合成负载参考值，不是成品验收。
   - 此前"原生回归待补跑"的欠账已清零；此后每轮改动照常按影响重跑对应项。

## 验证

- 最新 Rust core+transport **79 项**通过；无失败。前端 55 项与 `svelte-check` 0/0 为 2026-10-31 视觉单元结果，本轮前端未改动。
- 原生合成 `scripts/validate_chat_image.py`、`scripts/validate_screenshot.py` 及文本/胶囊/选区回归在视觉单元完成时全部通过；**隐藏窗口修复后的原生合成回归尚未重跑**——2026-11-01 修复时用户真实实例运行中，按约定不杀用户实例、测试会话也不路由到它；待用户退出实例后补跑，或直接以真实使用验收（选区期间无 Zwen 窗口遮挡即通过）。
- **真实环境（用户报告，2026-11-01）**：真实智谱视觉调用成功、收到完整回答；截图提问端到端可用。仅按用户描述记录，未读取正文/图片/元数据。修复前已知瑕疵：选区期间窗口遮挡；修复后待用户确认，若编辑/回答阶段仍有页面错乱则另立视觉单元处理。
- **不是完整真实验收**：多图、大图缩放画质、图片错误码路径、真实 Esc 取消、双屏/100%/150% 缩放、真实截图内存峰值、进程重启后的胶囊恢复、中文输入法均未验证。4MiB/6000px 保守限值待更多真实使用校准。
- 本轮未安装系统依赖、未增加常驻服务；未改动用户 Niri 配置。

## 下一最小任务

1. 用户退出 Zwen 实例后补跑原生合成回归（screenshot/chat-image/capsule/selection/chat）。
2. 用户真实验收"选区期间无遮挡"；若仍有错乱/半透明，按用户描述或截图定位布局/渲染问题另立单元。
3. 真实 Niri 交互截图的 Esc 取消实时归属、双屏与缩放表现，待真实使用继续积累。
4. 之后是 Markdown 知识库、AI 分类关联及可视化网络。没有自动采集当前应用上下文的需求。

每个小任务后查询 300 分钟用量；到约 95% 时完成当前最小单元、保存并停。当前环境没有该工具，无法给出实时用量。

16. **Markdown 知识库第一单元（2026-11-01 深夜）**：
    - 收藏：完整窗口回答区"收藏到知识库"——问题+回答结构化为 Markdown，截图以副本纳入，聊天附件不消费。
    - 知识库页：搜索（中文参数化子串）、列表（≤200 倒序）、查看（净化渲染）、两步删除、"重建索引"（文件为权威，采纳孤儿、清理缺文件行）。
    - 存储：`notes/<id>.md` + front matter + `notes.sqlite3` schema1 派生索引；临时文件→原子改名→索引事务；上限 256KiB/200B/6 附件；文件名白名单。
    - 验证：core 8 项新测试（全套 Rust 87）、前端 10 项新测试（全套 68）、原生 knowledge smoke 通过、**全套原生回归 9/9**；普通二进制 SHA256 `5afe66ea…3a392b`。详见 `docs/validation/2026-11-01-knowledge.md`。
    - 待真实使用：收藏一篇真实回答并重启验证。下一步：来源回看与人工编辑，然后 AI 自动整理。
17. **发送即清空、零字失败回填（2026-11-02 凌晨，用户反馈）**：
    - 用户反馈：模型思考中输入框还留着上一个问题；应为发送即清空，确认无法产生回答后问题才回到输入框。
    - 修复：发送被接受立即清空草稿（恢复原语义）；`settlePendingSend` 在第一个回答字到达时消费截图（不变），在**零字终态**（429/网络/停止且无任何回答）把问题文本回填输入框供一键重试；用户已开始输入新内容时不覆盖。
    - 前端 68 项、`svelte-check` 0/0；普通二进制 SHA256 `6488cb81d3fbb8c8f7c6633ea77d74470f5cd6d35f2f33e138af79559688eed4`。chat_image smoke 的失败回填断言与此语义一致；原生套件待实例空闲时例行重跑。
17. **知识库：来源回看与应用内编辑（2026-11-02）**：
    - 收藏记录 `request: <request_id>`；详情页"查看原始问答"跳转到历史详情（HistoryPanel 增加聚焦接口）。
    - 应用内编辑：标题+正文（Markdown），原子写回，身份/创建时间/来源/附件保留；`notes_update` 命令与权限新增；外部编辑器路径不变（重建索引采纳）。
    - Rust 89 项（+2：编辑保持身份/附件/时间、request 重建与非法值丢弃）、前端 70 项（+2：saveEdit 成功/失败）、`svelte-check` 0/0。
    - knowledge smoke 扩展：编辑流（改标题正文→界面与后端一致）+ 跳转流（断言历史详情显示原始问题）；原生套件因实例运行挂起等待，见 native-suite.log 追加段。普通二进制 SHA256 `71690fa1421f6adae8015be43b51838f039159a895fdea8bc5dadadba6dd2623`。
18. **知识库：来源回看与应用内编辑（2026-11-02）**：
    - 收藏记录来源 `request:`（front matter，校验与聊天记录一致，手改非法值重建时静默丢弃）；详情页"查看原始问答"跳转历史详情（HistoryPanel 聚焦接口；`load` 不再清除已选详情）。
    - 应用内编辑：`notes_update` 改标题+正文，原子写回，身份/创建时间/来源/附件保留；编辑器含取消与保存中禁用。
    - 验证：Rust 89 项（+2）、前端 70 项（+2）、`svelte-check` 0/0；knowledge smoke 扩展编辑流+跳转流，**全套原生回归 9/9（9 个 validate_*.py 脚本） 通过**（追加段见 native-suite.log）。普通二进制 SHA256 `ddd09430644310dcd2ea2a97573499e9cd6f83593423946b92ff74bf20304545`。
    - 调试记录：跳转后知识面板保留原详情视图属预期 UX，驱动初版误假设返回列表——"测试失败"两小时后定位为驱动假设错误，产品代码无此缺陷；教训与选择器一致：断言前先确认视图状态。
19. **AI 整理地基（2026-11-02）**：
    - `organize.sqlite3` schema1 + `OrganizeStore`：apply_batch（原子、changeset、范围强校验）、undo_batch（只删本批）、list_topics、note_ids、stale_assignments（note_hash 对比）、content_hash（FNV-1a，无新依赖）。
    - 管线：build_prompt（1–8 篇/正文 4000 字截断/总长 256KiB 上限）、parse_model_output（剥围栏、合并同笔记、丢未知与非法）、run_organize_job（既有 SSE 通路 + 密钥解析 + 单任务互斥 + 失败零修改）。
    - `organize_run` 命令注册（能力最小化照旧）；UI 与首次启用确认待下单元（UI 前命令不可达）。
    - Rust：core 4 项新测试（97）、desktop 6 项解析测试，合计 99 通过；`organize_run` 未接 UI 前无原生 smoke（下单元随 UI 一起加，含合成服务 429/坏 JSON 路径）。
20. **整理 UI 与首次启用确认（2026-11-02）**：
    - 知识库页新增"AI 整理"：勾选 1–8 篇 → **确认视图明示模型名、篇数与"正文将发送"**（每次整理都确认，满足"初次启用明确模型与范围"）→ 结果摘要 → **一键撤销本批**；主题条（名称+归属数）常驻列表上方，点主题看成员笔记并可跳详情。
    - 后端新增 `organize_topics` / `organize_topic_notes`（跨库联表取成员笔记）/ `organize_undo` 命令与权限；`TopicAssignments.note_ids` 改为序列化输出。
    - 修复真实缺陷：`notes.rs` 的 `TEMP_COUNTER` 误声明为 `const`（每次使用都是新原子，ID 计数器失效）——警告驱动发现，改 `static`。
    - 验证：Rust 93 项、前端 74 项（+4 整理流）、`svelte-check` 0/0；knowledge smoke 扩展整理服务（断言提示词含笔记 id 与"只输出 JSON"指令）与整理→撤销全流程（批次 undone、成员清空、空 AI 主题移除），连续 3 次通过；**全套原生回归 9/9（9 个 validate_*.py 脚本）**。普通二进制 SHA256 见 ordinary-binary.sha256。
    - 教训（重发）：`include_str!` 驱动改动后必须重建；驱动 `wait(check,label,limit)` 参数顺序曾致立即超时假失败。
21. **知识库收尾：来源元数据 + 导出 + 外链（2026-11-02）**：
    - 来源元数据：front matter `source_title/source_url/source_loc`（URL 仅 http(s)、≤2048；其余 ≤200 无控制字符），编辑器内补全，详情页展示（URL 点击走系统浏览器）；`notes_update_source` / update 集成；手改非法值重建时丢弃。
    - 导出：单篇与全部导出到 `~/Downloads/Zwen导出/`（0700，.md 原样复制）。
    - 外链：`open_external` 仅 http(s)、xdg-open 分离进程；渲染区链接点击统一拦截（`lib/links.ts` + 三处 prose 接线），webview 不再自行导航。
    - 验证：core 12 项 notes（+1 来源 roundtrip/拒绝/清除/重建保留）、前端 78 项（+2 links helper）、knowledge smoke 扩展来源编辑断言与导出（单篇+全部，文件内容校验）、javascript: URL 拒绝断言。
22. 单元状态：Unit A 完成。下一步：Unit B（主题改名/合并/拆分/重归类+关系+锁定+自动增量）→ Unit C（SVG 图谱）→ Unit D（自动划词浮条）。
23. **主题治理与关系 + 自动增量（2026-11-02）**：
    - core：rename_topic / set_topic_column / merge_topics（成员迁移、触及关系随删、目标锁定拒绝）/ split_membership（移动语义）/ delete_topic（成员+触及关系+主题单事务删除）/ set_topic_locked / add_relation(证据必须同时属于两侧主题)/ remove_relation / list_relations / auto_organize 开关（meta 表）。
    - shell：organize_rename/set_column/merge/split/set_locked/delete_topic/add_relation/remove_relation/list_relations/set_auto/get_auto 共 12 命令；UI 主题覆盖层加治理区（重命名/栏目/锁定/删除主题）与关系列表；整理面板加"自动整理新收藏"开关（持久化 meta）；收藏成功后 runAutoFor 静默增量。
    - 修复：organize_delete_topic 初版用哨兵合并写法被目标缺失阻断——改为专用 delete_topic；relations/meta 表首次建库重复声明去重。
    - 验证：core organize 5 项（+治理全链：改名/锁拒合并/合并删悬空关系/拆分移动/两侧行为证据关系/栏目/开关）；前端 80 项（+6）；knowledge smoke 治理段（rename/locked-skip/delete/auto 持久化）通过。
24. **知识网络 SVG 图谱（2026-11-02）**：
    - 阶段 5 落地：新"知识网络"标签页——主题节点（大小=归属数）、AI 关系为边（悬停看理由）；圆环布局（确定性，无第三方图依赖，设计文档的 Cytoscape 拟用改为手写 SVG 以避免新增依赖）；滚轮缩放/拖动平移/复位；搜索筛选（节点与边随查询收窄）；点主题节点按需展开成员笔记（≤12 个二级节点，幂等）；点笔记节点跳详情。
    - `lib/graph.ts`：buildGraph（过滤+圆环布局+边过滤）、expandTopic（幂等展开）+4 项测试；KnowledgeGraph 组件（a11y：节点 role=button+键盘，stage 交互注释豁免）；App 新 tab 接线。
    - 验证：前端 82 项（+4）、`svelte-check` 0/0；knowledge smoke 加图谱段（建主题→节点渲染→筛选→清空→恢复），全套原生回归 9/9（9 个 validate_*.py 脚本）。
25. **自动划词浮条 + Unit D 收官（2026-11-02）**：
    - 设置页新增"自动划词浮条"开关（持久化 `settings.sqlite3` schema1，启动时恢复）；开启后派生 `wl-paste --primary --watch <self> candidate-callback` 监听 PRIMARY——选中文字后自动弹出胶囊并以候选材料呈现（复用选区单槽收件箱，本地呈现，**不自动发送**）。
    - candidate-callback 入口（CLI + 单实例路由）；_mode=Candidate 与既有解释/翻译同路。
    - 验证：knowledge smoke 候选段（fake wl-paste 触发 + 开关持久化 + get_enabled）通过；**全套原生回归 9/9（9 个 validate_*.py 脚本）**；Rust 95 项、前端 82 项、`svelte-check` 0/0。
    - 边界：浮条位置当前跟随既有胶囊顶部锚点（不跟鼠标）；跨应用 PRIMARY 支持矩阵仍未验收；真实鼠标划词场景需真机验证。
## 数字勘误（2026-11-02 核对）

- 原生回归脚本为 **9 个**（desktop/capsule/selection/provider/chat/chat_fault/chat_image/screenshot/knowledge），此前若干段落写作"10/10"是笔误；以 `scripts/validate_*.py` 实际数量与 `native-suite.log` 为准。
- 前端测试当前实测 **82 项**（vitest 14 个文件）；此前"84 项"为笔误。Rust 当前实测 **95 项**。
- 各单元提交时的小计数字可能因当时统计口径不同而有出入；**审查时请以当前 HEAD 的实测输出为准**，并以 git 历史逐提交复核。

26. **接线审计与守卫（2026-11-02，面向外部审查）**：
    - 发现并修复：`notes_update_source` 在 build.rs 与权限中声明、却漏注册进 `invoke_handler`（前端只在编辑时合并保存来源，故未暴露）。已补注册，普通二进制重建为 `190d8176…2554cc`。
    - 新增 `src-tauri/src/main.rs::wiring_tests::commands_permissions_and_handler_agree`：解析 build.rs COMMANDS、capabilities allow-* 与 invoke_handler，三方集合必须相等；**已实测临时移除注册时该测试失败并点名缺失命令**，可作新增命令的强制门禁。
    - 新增面向审查 AI 的交接文档 `docs/AUDIT-HANDOFF-2026-11-02.md`（含逐单元代码地图、证据边界、自查失误、P0–P3 待审清单与复现命令）。
    - 勘误：原生回归脚本实为 9 个、前端当前 82 项、Rust 为 95（core+transport）+7（desktop）。