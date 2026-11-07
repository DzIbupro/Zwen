# 截图前隐藏失败及过期请求保护

本单元是截图残影排查中的确定缺陷修复，不等于完整动画残影修复。

旧attachment.run忽略hide_if_current的Result<bool>，无论隐藏失败还是intent已过期，都启动截图助手。现在仅Ok(true)进入capture；false取消旧请求，不重新唤窗；隐藏失败提示未开始截图，并通过原intent守卫恢复胶囊。预检失败只允许匹配pending.id的请求清理状态，避免迟到错误覆盖下一次请求或其完成提示。未更改模型、草稿、附件导入、截图等待上限和Esc行为。

验证：

- 新scripts/validate_capture_hide.py使用临时库、伪niri和smoke专用hide失败/过期注入，未截真实屏幕。旧版两种场景helperCalls均为1（RED）；修后均为0，草稿保留/无新附件/无模型请求（GREEN）。结果见assets/2026-11-06-capture-hide.json。
- Rust desktop51项通过，1项既有钥匙环测试忽略；新增迟到失败不清理新pending或完成notice的回归。
- 原有合成截图脚本通过附加/预览、禁止误发模型、重截、取消、超时晚到文件、清理、移除和重新发起。脚本原来仍使用旧Ctrl+Enter，初次卡在发送阻止检查，已同步为用户先前确认的Enter发送并重建include_str驱动后通过；不是本单元引入产品键盘修改。
- Vite、smoke和普通release通过，无新依赖；独立审查无可操作问题。产品前端未修改，不重复宣称本轮重跑173项前端。

普通SHA256 6a78609c083654030e915ab4db02356f44f48d91aab7dbeaf3b8e5d89ff51029，已原子安装artifacts/zwen，回退zwen-before-capture-hide-guard。普通文件检查不含FOUNDATION_SMOKE或ZWEN_CAPTURE_HIDE_FIXTURE_MODE。没有强关/启动日常实例，完整退出重开生效。测试不读生产库、进程环境、密钥，不请求真实模型或上传用户材料。

## 未完成的动画屏障

本机Niri26.04，当前配置使用默认window-close150ms；官方截图实现立即渲染当前场景取图。窗口hide返回/IPC窗口消失不能据此认定关闭动画已结束。已查[26.04截图源码](https://github.com/YaLTeR/niri/blob/v26.04/src/niri.rs#L1810)和[IPC类型](https://github.com/YaLTeR/niri/blob/v26.04/niri-ipc/src/lib.rs)，本次检查的IPC请求/事件没有可直接等待的动画结束通知。这里只记录调查，不声称全桌面API绝无其他办法。

后续应验证透明帧提交/呈现屏障或受支持的合成器机制，连同主窗/胶囊/已隐藏/中途重新唤出验证；不能仅加任意sleep、用GTK不可见或窗口列表消失冒充动画完成。原生鼠标/CUA工具未提供，不冒充真人截图验收。

剩余5项不减：历史删除；全选及8篇限制；取消选区清理；完整截图动画消退；无限等待及Esc/保存/子进程清理。周额度接近上限，本轮不另开复杂单元。
