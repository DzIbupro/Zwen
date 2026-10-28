# 本机验证摘要 · 2026-10-29

环境：Fedora 45 Server，Niri 26.04，GTK 4.24.1，PyGObject 3.58.1，wl-clipboard 2.3.0，约16GB内存。
两块屏幕，缩放分别为150%与100%。以下为一次原型验证，非完整产品验收。

| 检查 | 证据与结论 |
|---|---|
| GNOME Screenshot Portal | 版本2；interactive 请求 response=2；服务日志提示缺少 InteractiveScreenshot 方法。当前不选为默认后端 |
| Niri 截图 | 指定唯一输出文件后保存成功；收到包含相同路径的 ScreenshotCaptured 事件 |
| 命令完成语义 | screenshot 命令约62–69ms即返回，不代表图片已生成 |
| Esc 取消 | 用户确认操作；独立路径60秒观察期与最终检查均无文件。未验证程序自动识别取消 |
| PRIMARY | GTK原型中英文选区通过；GL轮次有一次失败，焦点干扰未隔离，跨应用仍待验收 |
| Gio 单实例 | 两次调用在同一主进程处理，第二次 remote=True |
| 中文搜索 | 默认FTS5词项匹配不能覆盖中文子串；首版采用参数化子串搜索 |
| 可用依赖 | GTK4、libsoup3、libadwaita已提供GI绑定；Gtk4LayerShell和GtkSourceView5未提供 |

| 原型渲染器 | 文本可见PSS | 隐藏稳定PSS |
|---|---:|---:|
| 默认（出现Vulkan日志） | 137.0MiB | 137.0MiB |
| GL | 129.1MiB | 129.2MiB |
| Cairo | 46.1MiB | 40.9MiB |

Cairo图片原型加载一张1280×720截图并追加50段文本：加载约44.8MiB，移除并隐藏稳定约45.2MiB。
PSS不包括完整GPU显存及桌面服务增量；未测帧率、能耗、20张图或30分钟稳定性。

待验收：Firefox与实际阅读器、中文输入法、多屏截图、连续请求、晚到事件、丢失事件、磁盘失败、真实模型、复杂排版和稳定性。
原始截图与日志仅保留本机，不进入仓库。
