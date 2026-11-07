# 独立选区操作条可行性实验（2026-11-03）

本轮是合成原型，未接入或部署到主程序。

1. scripts/spikes/selection_toolbar.py：本机GTK layer-shell小操作条319×44；keyboard mode NONE、exclusive zone0；显示前后原应用PID焦点相同。原型700ms后退出。按钮点击/真实选区集成尚未验收。
2. scripts/spikes/selection_geometry.py：只遍历脚本自己创建的测试PID，不读其它应用文本。系统toolkit-accessibility=false；默认运行未找到可访问选区。只给测试子进程设置GTK_MODULES=gail:atk-bridge和NO_AT_BRIDGE=0后，AT-SPI能返回选区offset13–22及矩形143,116,65,22。GI同名get_selection需显式调用Atspi.Text.get_selection。
3. 实际Niri客户端和服务都是26.04。平铺测试窗口tile_pos_in_workspace_view为null。将自有测试窗口浮动移动后，Niri给出500,343，而AT-SPI的SCREEN和WINDOW都仍为143,116。因此不能直接把所谓SCREEN值当绝对位置。浮动窗口可组合位置计算；平铺窗口不可猜测。selection_anchor.py 3项检查通过。
4. scripts/spikes/pointer_anchor.py：只建立一次短命透明layer-shell输入面，首个pointer-enter事件后销毁，超时200ms，不抓键盘、不记录点击、不截屏。实测返回本机监视器内鼠标位置，focusPreserved=true；最终重跑无绘制错误。第一次Cairo GI绘制转换错误已改为GTK透明CSS解决。这里只取得鼠标附近坐标，不是精确选中文字边界。原型非生产实现，还需验证是否会吞一次正在发生的点击/拖动、多屏与缩放、是否应改用更低层Wayland实现等；不能直接承诺无副作用。

下一最小单元：按“划词结束点附近的近似锚点”与“精确词句右上角”明确区分来源；验证一次性探针时机及去抖/取消，再让独立小操作条点击ID回传Rust，之后才激活主胶囊。不能在无坐标时默默弹屏幕居中胶囊。

上游资料：Niri WindowLayout文档 https://niri-wm.github.io/niri/niri_ipc/struct.WindowLayout.html 说明位置字段可选；上游PR4147 https://github.com/niri-wm/niri/pull/4147 讨论平铺窗口视口信息缺失。此处以本机实际输出为准。
