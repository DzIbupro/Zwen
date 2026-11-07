# 胶囊等待首字与单次弹出验证

## 用户反馈与根因

1. 原 Capsule 仅在response非空时绘制回答区域，running但尚无首字时没有等待状态。SelectionController成功ack还留下“选区操作已开始”，SelectionNotice据此显示“重新读取状态”，混淆了成功确认与错误恢复。
2. 原生toolbar_action等待windows::activate完成才派发selection-command；前端selectAction又通过switchSurface无条件set_surface。后者再次hide/show、重置初始高度与generation，构成重复呈现链路。

## 修补

- running且正文为空时显示等待模型提示和停止入口，首字到来后替换为正文；显式命令在选区确认尚未完成时也显示等待。失败状态继续走真实错误提示。
- 成功选区确认清空notice；失败确认仍保留原材料与错误。
- ensureSurface先读取原生mode，同模式只同步快照，不再调用set_surface；跨模式仍正常切换。原生层仍负责首次命令唤出，不削弱来源/选区身份校验。

## 结果

- 新增回归先观察4项失败（等待提示缺失、旧成功notice、同模式/跨模式协调），修补后前端144项通过，Svelte0错误0警告，Vite通过。
- 原生隔离actions通过：翻译/解释分别延迟首字1.5秒并分两段输出，等待提示存在、首字后显示正文；5次本机合成请求，保留旧历史，未发送/失败/运行中关闭和普通草稿保护等旧断言通过。
- 9个命令的surface generation增量均为1，覆盖问一问、翻译、解释、设置与文档动作。
- 既有capsule原生合成场景通过，固定宽度/增长上限/展开内容保留/旧resize拒绝未退化。
- 独立只读审查未发现阻断问题。
- 普通构建无FOUNDATION_SMOKE:/ZWEN_SMOKE/BAR_TRACE/ZWEN_ACTIONS_FIXED_ANCHOR；已安装并启动artifacts/zwen。SHA256：4060c47d97d884d6766e3cd5cc81f04f6372d4219befefa141dd9ad7245fdc17。

## 证据边界

首次原生driver在进入待修场景前遭遇200ms实际指针探针超时。为使UI流程验证独立于真实鼠标位置，新增desktop-smoke特性下显式ZWEN_ACTIONS_FIXED_ANCHOR=1夹具，输出anchorMode=synthetic-fixed；正式定位逻辑未改，钩子不编译进普通程序。该测试证明自身GTK/WebKit流程和窗口generation次数，不证明所有真实页面划词/实际指针定位或用户肉眼动画全覆盖。

未读取生产数据库、进程环境或密钥；未自动请求真实模型、采集屏幕或上传用户材料。此修补改善等待与呈现，不证明真实模型服务已恢复或响应速度已加快。
