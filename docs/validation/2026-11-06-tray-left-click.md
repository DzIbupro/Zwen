# 托盘左键激活修复

用户要求左键直接打开主界面，右键继续显示菜单。本单元仅修此项，不改现有保存、划词、截图和模型调用语义。

## 原因与实现

本机锁定tray-icon0.25.1的默认libappindicator后端不实现SNI Activate处理，且Tauri文档明确Linux不支持show_menu_on_left_click开关，单纯加Click回调无效。真实旧版隔离回归得到“No handler for Activate”。

启用同一版本tray-icon的ksni特性（Linux目标依赖，Cargo特性合并选择KSNI后端）；锁定ksni0.3.7，锁文件增加ksni及4个小型依赖，其余既有版本不变。现有Tauri菜单和默认圆角图标仍复用，右键菜单保留。只响应Left/Up点击，复用zwen.tray.open→LaunchAction::Show，不另建唤窗或直接exit路径。不增加常驻外部进程或系统软件。

## 验证证据

- 172项前端测试通过；Svelte0错误/0警告；Vite通过（原有分块体积提示）。
- Rust完整工作区189通过，1项既有真实钥匙环测试忽略；普通release通过。
- 新脚本scripts/validate_tray_activation.py：临时应用数据、真实会话org.kde.StatusNotifierWatcher及Niri窗口；旧版失败、新版通过。检查ItemIsMenu严格为false、隐藏后真实D-Bus Activate唤回并聚焦、DBusMenu含5个操作、菜单“打开”唤回、菜单“胶囊”切换及Activate再展开、菜单“保存并退出”走正常退出并清除注册项。结果见assets/2026-11-06-tray-activation.json。
- 当前安装DMS SystemTrayBar.qml横/纵托盘与溢出区均在onlyMenu=false时走trayItem.activate，右键走DBusMenu。本次实际调用并验证的是该原生协议，不是直接调用应用dispatch。
- 隔离原生foundation edit/restore/quit三阶段通过：脏草稿关闭保存、隐藏恢复、跨进程恢复和最后未保存追加文本通过X保存退出。短回归进程树PSS峰值346.18MiB，仅本次测试样本，不代表长期或全部场景上限。
- 独立代码审查无可操作问题。

原生CUA/鼠标控制工具未提供，故未声称真人左键/右键像素验收；当前桌面宿主协议和Niri实际窗口行为已验证，实体鼠标试用仍待用户确认。未读生产数据库、进程环境或密钥，未请求真实模型或上传材料；测试仅临时合成数据。

普通SHA256：9675ae632a5491423e9e9f2a6cd55d3e7ba37d7c35f9d4e050e6722b1cea4475。安装artifacts/zwen，原子替换且保留zwen-before-tray-left-click；不强关或启动日常实例，完整退出再打开生效。
