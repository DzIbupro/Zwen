# 侧栏和系统主题验证（2026-11-05）

- Logo原问题来自collapsed时justify-content:center瞬时作用于尚未收缩的宽度；保持固定左边距、左对齐及品牌容器不收缩。
- GSettings监听复用gtk::gio，无新增依赖。DMS源码/usr/share/quickshell/dms/Services/PortalService.qml:setSystemColorScheme将浅色写default、深色写prefer-dark。只监听该键；system作为用户偏好保存，渲染采用解析后的浅深值。手动选择不随系统改变。原生窗口和划词条刷新同步；前端监听在卸载时清理。
- 前端166项29文件通过，Svelte0错误0警告，Vite通过；新增system测试在实现前失败（原parse拒绝system），实现后通过。
- Rust desktop50通过/1既有忽略；普通release成功，SHA256 8c1e0e3f593f53c5cceb9d9652b4851849422b135cc46bd06bc3087d84b199a1。
- validate_system_theme.py通过：两个原生进程，临时SQLite，GSETTINGS_BACKEND=memory。真实Gio changed信号到WebView：深→浅→深；显式浅色不被系统变化覆盖；重新选system即时匹配；system重启持久化；系统变化后渲染器reload仍正确。Logo收起动画24帧坐标全为20px，展开后保持原位置。原始记录见assets/2026-11-05-system-theme.json。
- validate_appearance.py原有两进程恢复通过：深色、收起侧栏及草稿保持。无模型请求。
- 开发中发现测试驱动误用了不存在的JS WebviewWindow.reload，改用location.reload重建驱动后通过；不属于产品运行错误。重载时系统初始化值可能过期，load_appearance额外同步当前系统缓存，已有原生reload断言覆盖。
- 浏览器共用前端检查三态设置，截图assets/2026-11-05-system-theme.png。原生测试由应用内隔离驱动完成，不冒充真人鼠标或实际桌面开关操作验收；没有改变用户桌面配色。
- 原子安装并保留zwen-before-system-theme；不主动关闭/打开日常实例，完整重开后生效。其他待办不在本单元范围。

独立审查发现Gio连接后须首次读取以及分离原子缓存的竞态，已补齐读取并改为单一原子主题模式、按需解析系统值；最终桌面回归、Rust测试和普通构建重跑通过，审查复核无重要遗留。
