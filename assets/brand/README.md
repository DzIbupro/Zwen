# Zwen 标识

原图由用户于2026-11-02指定，保存为`zwen-logo-original.png`；保留原有黑底白色图形，不重绘、不裁剪。

派生展示文件仅做等比尺寸转换与RGBA编码：
- `frontend/assets/zwen-logo.png`：96×96，主窗口、胶囊和浏览器图标共用。
- `src-tauri/icons/icon.png`：256×256，Tauri窗口/桌面识别图标。

这些文件应随源码保存；build.rs不再生成或覆盖图标。桌面入口模板见`packaging/zwen.desktop`。
