# 截图生命周期修复计划

用户已确认：Zwen管理框选，松开鼠标保存、Esc取消、支持持续等待；每次修复推送持续授权。

1. core截图后端改用固定slurp/ grim：选择等待无默认期限，退出码取消/成功为结束信号；geometry严格验证，grab仅有限故障超时；owned process group取消回收及Linux父进程退出保护。先写合成测试，覆盖取消、长期等待、非法几何、文件失败和后代清理。
2. 主窗截图前提交透明帧，确认Wayland对应帧回调后隐藏，超时/过期不启动截图；任何分支恢复原Widget状态。先在独立合成GTK窗口验证，不读取真实桌面。
3. 接入取消/重触发/退出所有权，更新合成原生驱动（不截真实屏幕）。Rust/前端检查、独立审查、普通构建安装、记录证据及限制、ff-only回开发分支、推送既有私有PR，不合并main。

透明帧探针结果：240x120合成窗口alphaMax0，frame3 complete，有presentation timestamp；这仅证明GTK帧机制，不等于最终合成截图无残影。来源：GTK3 frame callback实现及Niri26.04截图源码，详见验证文档。

## 完成记录（2026-11-08）

核心进程所有权、透明提交ACK与窗口锁串行、退出自动取消/半写清理均实现；217 Rust、188前端、长61.25秒、原生生命周期/退出/隐藏保护/划词回归及隔离Niri1.5倍缩放三场景像素通过。FrameTimings.complete路线被实际测试否定，改为同连接异步sync；详见validation/2026-11-08-capture-lifecycle.md。
