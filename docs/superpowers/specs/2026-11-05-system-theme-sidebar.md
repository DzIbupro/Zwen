# 侧栏Logo定位与跟随系统（2026-11-05）

用户明确要求：收起侧栏时Logo原地保留，不向右跳；外观增加“跟随系统”，桌面主题与配色中的浅色模式打开则浅色，关闭则深色。手动浅/深模式保持原行为，保存用户选择，不自动修改现有主题偏好。

定位根因：collapsed样式切换justify-content:center和padding，在轨道宽度开始收缩前把Logo居中到旧宽度。采用固定左边距/左对齐和不收缩品牌容器，保留原轨道动画。桌面DMS PortalService.setSystemColorScheme把浅色模式写为GSettings org.gnome.desktop.interface color-scheme default，深色写prefer-dark。监听这个现有设置，不读桌面私有状态文件、不轮询、不新增依赖。浏览器预览以prefers-color-scheme降级。

主题偏好增加system值，实际渲染仍为light/dark；原生窗口与划词条一起更新。页面重载从当前系统缓存重新同步，避免初始化脚本旧值。切换仅改变外观，不激活或重定位窗口。使用临时SQLite及GSettings memory后端验证，不改用户桌面配色。

验收：Logo动画逐帧横向不变；三态可保存；系统明暗实时联动；显式浅/深不受系统变化覆盖；重启与渲染器重载正确；已有聊天、草稿及模型设置不受影响。
