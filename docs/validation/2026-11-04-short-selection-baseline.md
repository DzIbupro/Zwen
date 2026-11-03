# 2026-11-04 短划词真人基线：定位失败，尚未修复

## 结论与范围

用户反馈：本轮已试过的短选区均没有小操作条；**长句未测试**，不能记作长句失败。夹具计数为 11 次有效拖选、0 次小条出现、0 次检查按钮按下/点击。此次夹具仅提供自有合成文字的实际选区，未读取用户文档、未调用模型。

失败点已收敛到原生定位阶段，而非最短文本长度：11 次拖选的最终 probe 全部未收到 enter，均在按住拖选期间到达 200ms 上限，之后读取到非空 PRIMARY，show 因锚点已经失效被拒绝。现场行为与源窗口在拖选期间持有 Wayland 隐式指针抓取一致；这是机制解释，不把未采集的协议事件当作已观测事实。

本轮只诊断，**未修改生产定位**。不能宣称短选区已经修复，也不能把汇总出现次数或合成核心测试替代真人可见验收。

## 原始证据

原始基线日志已备份至 `artifacts/validation/short-selection-baseline/`。`fixtureSelectionId` 是夹具事件 ID，和 Zwen 内部捕获 ID 不同；跨组件按单调时钟、payload hash 和长度关联。应用 `timeNs` 是日志行接收时间，不是 compositor 的协议时间。

| 文件 | SHA-256（本次基线） |
| --- | --- |
| `bar-input.json` | `a71238101dc4145f87ce1eb483e90e27d3d22f7ea2a742c560466eb849291f63` |
| `bar-input-fixture-events.jsonl` | `66f01a23d97db6571c0f597d5e5a9aa263e4e9e24d67a57d3e55e96b16058f60` |
| `bar-input-helper-events.jsonl` | `1f7b8065bcefe98194e24b2d900f8e3a7b7344603630a5bd5f365bff032a52ee` |
| `bar-input-app-events.jsonl` | `18f3fc609a714f84e3f6863ae74f378f7c5be2f22a8d3f941f6d196c45f2e3a5` |
| `bar-input-stderr.txt` | `da3320741310a6eb206fadfe35838980f5c4f2fafae26edc6852b94eeaece8ce` |
| `bar-input-stdout.txt` | `cdd9c8cc3af53a528cd3e3c0494fcdeec4eb2038fb373f85215a7fb024f8ca39` |

后续重跑夹具会覆盖同名输出，应先保留本轮文件并核对上述 hash。

## 逐次拖选关联

每行取一次 `text-release` 且 `moved=true` 的事件，以及它之前最后一次 prime。字符数包含选区里的空白；最终实际选区为 2–10 个字符，过程中也记录了单汉字和单字母的选区变化。最后一行只选中了短英文片段，不是长句覆盖。

| 夹具最终 ID | 字符数 | Zwen 最终 prime ID | prime→release ms | timeout→release ms | enter | show accepted |
| --- | --- | --- | --- | --- | --- | --- |
| 3 | 2 | 3 | 306.2 | 106.3 | 无 | false |
| 6 | 2 | 6 | 1456.9 | 1257.5 | 无 | false |
| 9 | 2 | 9 | 1193.3 | 993.8 | 无 | false |
| 13 | 3 | 13 | 292.3 | 92.9 | 无 | false |
| 16 | 2 | 16 | 1550.2 | 1350.3 | 无 | false |
| 19 | 5 | 19 | 677.1 | 477.4 | 无 | false |
| 24 | 4 | 24 | 420.0 | 219.7 | 无 | false |
| 29 | 4 | 29 | 970.6 | 771.4 | 无 | false |
| 32 | 2 | 32 | 890.9 | 690.9 | 无 | false |
| 35 | 2 | 35 | 381.0 | 180.9 | 无 | false |
| 46 | 10 | 44 | 510.8 | 310.5 | 无 | false |

全部 timeout 对应当时 `src-tauri/src/selection_bar.rs:206` 的 200ms probe 清理路径。

例：id6 在相对时间 55746.9ms prime，55946.2ms timeout，55968.6ms helper 读取到 2 字符选区，56003.9ms show rejected，57203.7ms 用户才松开鼠标。PRIMARY 有效，但 probe 早于松手约 1257ms 消失。

## enter 与旧 prime 清理的含义

45 次 prime 仅有 9 次 enter：id4、7、10、14、17、20、25、30、45。它们均对应清空旧选区的普通点击及随后的鼠标释放；多数随后读取到空 PRIMARY，按设计走 `selection.rs` 的取消路径，再由 `selection_bar.rs:110` 的 discard 清理。因此这些 discard 不能单独解释为“有效短选区被错误取消”。

唯一 `show accepted=true` 为 id17：空选区事件 prime 后先收到 enter，200ms 读取时用户已经开始下一选区并产生“中文”，随后新 id18 prime 替换旧表面。该记录没有对应 `BAR_INPUT_OBSERVED`，不能算小条可见成功。

同次拖选频繁 PRIMARY 更新确实会替换旧 prime，增加探针抖动；但它不是完整根因。即使最终 prime 完全不被新 ID 替换，本轮每次松手仍在其 200ms 生命周期之外。只修复旧 prime 继承、修改字符门槛或调整文本防抖不足以覆盖已测的按住 1 秒拖选。

## 代码路径与已有验收

`src-tauri/src/selection.rs` 在候选读取前 prime，然后等待 200ms，再读取 PRIMARY。`src-tauri/src/selection_bar.rs` 同时给 probe 200ms 生命周期；只有 enter 捕获可靠 monitor-local 坐标后才能 show。Wayland 下拖选期间 enter 未到达，这两段时序使最终有效文本到达时已没有锚点。

`crates/core/tests/selection.rs` 已加入单汉字“中”和单字母“a”的 helper 读取→候选预览→冻结验收。命令 `CARGO_HOME="$PWD/.cache/cargo" cargo test --offline --release -p zwen-core --test selection` 结果 21 passed。该结果证明核心没有最短长度门槛，不证明 GUI 定位成功。夹具/内嵌 helper 编译、精确 payload/长度/hash 与无窗口 Gtk.TextBuffer mark-set 验证也通过。

## 替代能力只读核查与后续边界

- 本机 `niri msg --help` 无 cursor-position 查询。没有执行 pick-window/pick-color 等交互操作。
- 本机 `/usr/share/gir-1.0/Atspi-2.0.gir` 和 [AT-SPI EventListener.register 官方文档](https://gnome.pages.gitlab.gnome.org/at-spi2-core/libatspi/method.EventListener.register.html) 列出 `mouse:b1r`（左键释放）以及 mouse:abs/rel。**事件名称存在不等于本机 Niri/Wayland 会可靠发出**；本轮未注册监听器、未启用全局采集，不能把它当作可用的修复渠道。后续若获准，可仅用自有夹具验证释放事件的来源、坐标、可用性与撤销生命周期。
- [InputCapture 官方接口](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.InputCapture.html) 是经会话及授权的输入捕获机制，激活后输入交给应用，不能直接当作被动监听替代。本机 niri portal 配置默认 gnome/gtk，gnome 声明 InputCapture 接口；仅静态声明不能证明 Niri 后端运行能力。本轮没有创建会话、弹权限框、改 portal 设置或捕获输入。
- [RemoteDesktop 官方接口](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.RemoteDesktop.html) 的 pointer 方法用于发送输入，不能据此推断可被动收到其他客户端的释放事件。

不以延长全屏透明 input probe 或重复重建探针来掩盖失败，也不在延迟后采样用户已经移到其他按钮的位置。有可靠选区附近坐标时维持 near-selection 行为；拿不到可靠位置时是否允许显示固定上方居中候选条，需要用户明确选择，基线采集时未实施。来源窗口、候选 ID/代次、禁用与失效撤销约束应在任何后续方案中保留。


## 基线之后的已批准实现（待真人复验）

用户随后明确批准：没有可靠选区附近坐标时，独立小条固定在当前阅读屏幕上方居中；可靠坐标继续靠近选区。实现已加入 `selection_bar.rs`/`selection_bar_geometry.rs`/`selection_bar_router.rs`：probe 到 200ms 后先撤销输入gate、同步销毁全部透明输入表面，再保留同 ID 的 expired 锚点供有效候选降级。过期锚点拒绝任何后来指针采样；新选区、来源变化、禁用和失效仍沿用原候选/lease约束。

固定位置优先匹配 niri focused-output 的 logical 矩形与 GDK monitor；无法唯一匹配时明确采用 primary-monitor 降级（GDK 无 primary 时取第一个可用 monitor）。诊断 `anchorKind` 分别为 `pointer-proximity`、`synthetic-fixed`、`fixed-top-center-reading-output`、`fixed-top-center-primary-monitor`（GDK未提供primary时明确为`fixed-top-center-first-monitor`），未显示时为 `unavailable`，不把固定条宣称为 near-selection。

纯 Rust 几何测试完成 RED→GREEN（11 passed），覆盖超时后固定位置、拒绝迟到坐标/错误ID或PID、可靠早期坐标保留、不同逻辑屏幕/缩放匹配和小屏边界。此实现尚需原生集成检查和用户真人短选区复验；以上 11 次失败仍是修补之前的真实基线。
