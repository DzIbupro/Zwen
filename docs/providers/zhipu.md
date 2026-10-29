# 智谱接入记录

用户选择（2026-10-30）：

- API根地址：`https://open.bigmodel.cn/api/paas/v4`
- 当前文本模型：`glm-4.7-flash`
- 后续截图/图片模型：`glm-4.6v-flash`（附件通路尚未完成，不声称已能看图）
- 密钥环境名：`ZWEN_API_KEY`，密钥值不进入SQLite、前端IPC、仓库或日志。

官方依据：[GLM-4.7-Flash](https://docs.bigmodel.cn/cn/guide/models/free/glm-4.7-flash)、[GLM-4.6V-Flash](https://docs.bigmodel.cn/cn/guide/models/free/glm-4.6v-flash)。聊天POST为根地址后加`/chat/completions`，Bearer鉴权。模型的reasoning_content目前不展示、不持久化，正常回答content走Markdown渲染。

文本请求/取消/历史/多轮已用本机合成服务验证；真实智谱服务必须单独验收，不能把合成测试当成实测。暂不自动添加工具调用、联网搜索或复杂推理参数。

## 在本机提供密钥

普通程序构建好后，在你自己的终端执行：

```bash
cd /home/dzibupro/Zwen/Zwen
./scripts/run-with-key.sh
```

脚本先请求旧实例保存退出，再以不回显方式读取密钥，只传给新应用的进程环境。不会创建.env文件，不要把密钥发到聊天里。首次配置的非敏感地址/模型通过core配置示例写入；已有配置保持不变。

打开模型设置确认地址、模型与“检测到密钥”。可用一句不含隐私的测试问题验证真实请求。如失败，保留窗口里的错误类型；不要把密钥加入截图或日志。

## 首次真实服务验证

用户已在本机提供密钥并测试，智谱返回HTTP429，首次未得到成功回答。用户随后重启新版并手动重试，最新元数据确认glm-4.7-flash完整结束、无错误并收到3字符回答，真实文本接入已通过一次。前一次429的具体原因未知。429不能一概当并发限制：官方错误码还包含账户/额度/Key类型等原因。当前软件已增加最多8KiB、2秒有界错误体读取，只提取最多6位数字错误码，不回显原始message，不保存完整错误体，也不自动重试。后续若再失败可通过数字码定位；本次重试成功，不再自动请求。依据：[官方错误码](https://docs.bigmodel.cn/cn/api/api-code)、[速率限制](https://docs.bigmodel.cn/cn/api/rate-limit)。

官方区别：1302是用户速率限制，1305是模型平台过载；1113/1308/1310/1315等另有账户、额度或Key类型含义。没有具体码时不推断原因、不要求充值。新版需重启后再做一次手动测试。
