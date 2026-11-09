# 论文页图补读与资料库分页

用户确认对话入口可用后授权继续未完成项。本轮继续 `feat/paper-analysis-v2`，未合并主分支。保留既有输入区小卡片，不增加独立论文导航。

## 已实现

- 默认仍按Poppler文字→内置skill→Markdown分析。配置现有图片模型后，在“阅读选项与资料→阅读要求”可启用所选页图补读，最多8页。无文字扫描件提供快捷启用入口；没有配置图片模型时明确说明。
- 沿用Poppler `pdftoppm`，按PDF物理页逐页渲染为PNG data URL，通过现有兼容图像消息发送。每页最长20秒、最长边1600像素、PNG最多5 MiB，整次渲染最长120秒；发送前完整解码检查PNG。图像只驻留本轮内存，不写入聊天记录或额外页图文件。
- 渲染在现有原生任务worker内完成，复用取消、保存、错误与停止生命周期；不在启动锁内等待。既有截图裸Base64路径保持兼容。
- 会话保存readImages与范围，每次追问重新提交同范围原文/页图。当前模型、页码及再次发送图像的提示可见；导出也准确标识读取方式。原文核对区明确只展示本机文字提取。
- 资料库schema1→2：事务新增元数据索引，列表按50条游标分页，恢复草稿按id查询，不为列文件加载全文。原件、来源及会话保留；旧记录兼容默认readImages=false。
- 升级前生成私有SQLite一致快照，以逻辑内容摘要命名，校验quick_check、schema及原内容，完成后原子保存。同内容升级失败重试复用同一备份，失败暂存文件清理；不反复复制大库。未打开或迁移用户生产库。
- 临时附件查询失败保留自定义要求及附件id并可重试；未配置图片模型时仍可关闭已经勾选的选项；历史范围未加载前阻止发送。

接口格式参考官方[阿里云视觉模型文档](https://docs.bailian.console.aliyun.com/zh/model-studio/vision)及[GLM-V官方示例](https://github.com/zai-org/GLM-V/blob/main/skills/glmv-caption/scripts/glmv_caption.py)。这不是对所有兼容服务实际图像能力的保证。

## 验证

- Rust workspace：237通过、0失败、2原有忽略；原生65通过、1原有忽略计入总数。覆盖扫描页、超过8页、PNG完整性、运行中子进程取消、旧库>1 MiB记录迁移与快照、失败升级重复打开只留一份备份、分页不解析损坏正文及按id恢复。
- 传输测试先复现PNG data URL被拒绝，再修复通过；保留裸Base64回归，并拒绝空data URL、外部网址及非PNG data格式。
- 前端208通过；Svelte 0错误/0警告；Vite构建通过，仍有既有大chunk提示。
- 隔离Weston/pixman原生闭环：原生选择器、普通对话文字分析、自定义要求、追问附原文、选区关闭保留草稿、部分停止保存；再启用图片模型做页图分析和追问。共5次本机合成请求，检查3张PNG data URL、模型选择、原文及skill。结果 `artifacts/validation/paper-v2/result.json` 的pageImages=true、realModel=false。截图已查看，位于同目录chat-composer.png；软件合成器的透明层表现与日常桌面可能不同。
- 测试仅用合成PDF和临时库，未发送用户真实论文或调用真实模型。独立代码复核发现的会话提示、导出、备份重试、选项关闭、附件恢复和游标查询问题均已修正并回归。

## 交付与边界

普通release构建通过，检查不含论文测试钩子。已原子更新 `artifacts/zwen-paper-analysis` 与 `artifacts/zwen-paper-analysis-v2`，SHA256为 `50115b8bde213dc788a3850373034f8dd6fef13c341eecf019b9735b1fb093e4`；前版备份 `artifacts/zwen-paper-analysis-before-page-images-20261109`。

日常 `artifacts/zwen` 未改，SHA256仍为 `91a02db43068ef9bd3c918386a3a6f16e85d8c6f8f84584c81f1cb45dd15878a`。未强退用户实例，完整退出旧试用程序后原命令 `./artifacts/zwen-paper-analysis show` 生效。

原生PDF附件协议、自动agent式长文读取和独立本地OCR尚未实现；超出文字预算或页图8页范围仍需选页。真实图片服务的权限、上下文/图像配额及内容准确性未实测，用户先前授权的2次真实调用已用完，本轮没有追加请求。

## 原入口补交付（2026-11-09）

用户报告找不到入口。核对本机 `~/.local/share/applications/zwen.desktop` 的Exec仍为 `artifacts/zwen show`，而前述交付只更新论文试用产物；桌面入口因此继续加载旧程序。现将已验收、无测试钩子的同一普通构建原子安装到 `artifacts/zwen`，安装后校验SHA256与试用版一致：`50115b8bde213dc788a3850373034f8dd6fef13c341eecf019b9735b1fb093e4`。旧日常程序备份为 `artifacts/zwen-before-paper-integration-20261109`。未修改桌面配置、源码分支或用户数据，未强退用户实例；运行中旧实例需完整退出重开，尚未宣称用户日常窗口已显示新版。入口在主窗口“对话”的输入框上方。
