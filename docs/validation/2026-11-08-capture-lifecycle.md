# 截图残影与等待/取消生命周期修复（2026-11-08）

## 用户决定与产品行为

用户明确选择：**Zwen 管理框选，松开鼠标保存、Esc 取消、支持持续等待**。因此生产路径不再调用 Niri 内置截图UI：固定 slurp 框选、grim 写图；已安装这两个组件，不新增系统安装。选择期间无默认截止时间；实际图像写入单独设5秒故障超时。截图仅附加草稿，不自动调用模型。取消、重新触发和退出会回收本应用拥有的截图进程组。

关闭应用会自动取消正在等待的截图，保留既有未保存编辑、选区和文档输出保护；不再因截图 pending 要求用户先手动取消。退出先撤销导入所有权、关闭进程注册表、终止组并有界确认直接子进程退出，再清理私有 incoming 中的请求文件，不删除已保存附件。Linux子进程也设父进程退出保护；主动取消同时杀组并回收leader。waitid(WNOWAIT)使leader在组回收前不被reap，防PID/PGID复用误杀。几何严格验证，参数独立传递；过大、非法、非零退出、部分文件均不能附加。旧结果仅可更新同一pending/settled ID，不能覆盖新截图提示。附件列表和pending状态共用提交锁，避免旧列表/新idle不一致。

## 透明缓冲与原生窗口所有权

主窗先隐藏子widgets、临时拦截根draw并清空RGBA，再确认同一GDK Wayland连接已处理缓冲提交，之后才unmap。异步 wl_display.sync 监听器完全留在GTK线程，超时/过期/失败均恢复原draw处理、可见性和app-paintable；不在GTK线程做阻塞roundtrip。隐藏准备与旧窗口操作在geometry锁下串行，GTK回调不拿geometry锁，避免旧激活在透明准备后又画回内容。普通激活/隐藏也使用该机制，防先前关闭快照污染“已隐藏”状态。X11跳过Wayland屏障，维持原窗口行为；新框选功能面向Fedora/Niri/Wayland。

最初等待GdkFrameTimings.complete在普通GTK探针可行，但实际WebKit全透明窗口被合成器剔除后可能没有呈现回调。该方案未通过像素测试，已改为提交后的异步服务器确认，再以实际隔离截图验证内容。不是固定sleep，也不以窗口列表消失证明无残影。参考[GTK Wayland实现](https://raw.githubusercontent.com/GNOME/gtk/3.24.50/gdk/wayland/gdkwindow-wayland.c)、[FrameTimings语义](https://docs.gtk.org/gdk3/method.FrameTimings.get_complete.html)、[Niri26.04快照实现](https://raw.githubusercontent.com/YaLTeR/niri/v26.04/src/niri.rs)。

## 证据

机器安装niri26.04。结构化结果见[原始合成记录](assets/2026-11-08-capture-lifecycle.json)。

- Rust工作区217通过、0失败、2默认忽略：既有钥匙环1项；另1项是显式61秒等待回归。后者已单独执行，**61.25秒后仍保存成功**。
- 前端188通过、Svelte0错误0警告、Vite构建通过。普通release及不含smoke检查另记安装指纹。
- `validate_capture_pixels.py`：另启不导入全局会话环境的嵌套Niri，只含固定#123456背景和全红合成Zwen。关闭动画延长为1500ms，真实grim只读取这个隔离桌面；主窗/胶囊/已隐藏三场景截图中非背景像素均0。最终actualScale=1.5，所有透明根alphaMax0且Wayland sync已确认，每次截图后红色UI都恢复。早期默认scale1三场景也均0；不把隔离桌面证据当成真实用户桌面/所有合成器长期验收。
- `validate_screenshot.py`：假slurp/grim+合成PNG，附加/预览/禁止误发模型/重截/主动取消/Esc/测试故障超时/清理/删除/重触发全流程通过，孤立incoming文件0。生产没有测试等待截止时间。过期intent的透明准备被拒绝属于预期保护，不强行要求旧请求完成。
- `validate_capture_hide.py`：失败/过期注入均0截图助手调用；已为新slurp/grim补假助手，防回归时触碰真实桌面。
- `validate_capture_exit.py`：写入器先创建半写文件再等待；通过实际quit入口退出，写入器终止且partial文件删除。原先退出被前端pending检查阻塞，先复现失败，再修复通过。
- `validate_actions.py`：全局隐藏/激活变更后，完整合成划词、其他/设置、来源窗口切换、胶囊、失败重试和草稿保护通过；所有8个请求仅本机合成服务，文档只写/tmp。
- 新核心测试覆盖负坐标、非法/过大几何不调用writer、无用户等待上限、取消继承管道后代、writer部分写入取消、应用shutdown后拒绝新工作；新附件测试先复现旧完成覆盖新提示再修复。独立审查/复审无重要遗留发现。

## 边界

测试未读取生产库或真实PRIMARY/CLIPBOARD，未探查日常Zwen或其他生产进程的环境/密钥；fixture仅继承启动环境并注入测试变量，不输出环境。没有真实模型调用或个人材料上传。原生GUI、GTK/DOM/IPC与隔离像素测试不等于实体鼠标/键盘人工验收。Esc语义对应已安装slurp手册；实际键盘按下和框选释放仍待用户日常试用。旧Niri capture_with保留为兼容测试适配器，生产入口只走新后端，不以遗留常量60秒代表现行行为。

## 安装

普通SHA256 91a02db43068ef9bd3c918386a3a6f16e85d8c6f8f84584c81f1cb45dd15878a。原子安装artifacts/zwen，备份zwen-before-capture-lifecycle；不含FOUNDATION_SMOKE、测试超时或像素探针代码。不关闭/启动日常实例，完整退出重开生效。无数据库迁移或Cargo新依赖；新增直接Wayland客户端ABI链接，使用GTK会话已有库。
