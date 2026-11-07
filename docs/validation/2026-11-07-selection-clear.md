# 取消选区自动清理验证（2026-11-07）

## 修复与边界

根因是外部空PRIMARY读取经过cancel_read恢复previous_candidate，操作条虽然可能隐藏，收件箱仍保留旧材料。新增Inbox::clear_candidate_read，仅当前未完成candidate ID能清除自身及旧预览；已冻结/显式导入/新一代材料不受影响。

候选读取仍在前后核对完整FocusSource(pid,window_id)、watcher token和当前read ID。仅稳定外部来源的空/不可用文字读取采用清空路径；自己或未知焦点仍保持抑制回滚，避免自己窗口事件丢失材料。旧epoch只能撤销对应旧操作条，不可撤销新呈现。

为避免清空与点击命令竞态，GTK左键按下时即绑定操作：在Inbox+Ledger锁内冻结匹配材料并撤销匹配epoch，之后才做后台来源验证和窗口激活。同一epoch不能重复派发；设置按钮清除候选但不冻结正文；来源验证失败只确认自身ID。点击派发兼容合成GTK激活；原生指针交互采用按下触发（不等待松开）。其他菜单仍为展开操作，不自动发模型。

移除SelectionNotice的手动丢弃、重新读取按钮与导入/丢弃成功提示；保留真实错误及失败重试。手写草稿不在清空路径修改。审查发现discard(false)会误清真实错误，已以失败回归复现并修复，静默确认保留原错误。

## 验证

- TDD：新增2条核心clear测试和2条Ledger绑定测试，缺API失败后实现通过；前端无成功提示及错误保留均先观察旧行为失败再修复。
- Rust工作区209通过、0失败、1既有钥匙环忽略。
- 前端33文件188测试通过；Svelte0错误0警告；Vite通过（既有chunk提示）。
- validate_actions.py：临时PRIMARY/source/anchor及本机合成服务，原生操作条完整回归通过；强化cleared-selection阶段要求claim_selection.item为null。问一问、翻译、解释、其他、设置、同PID不同window、来源竞态、草稿所有权、失败/部分正文/关闭及文档临时输出均通过。
- 原生锚点为synthetic-fixed；测试是DOM/GTK驱动及IPC，不冒充实体鼠标或长期验收。没有读取真实PRIMARY、生产库、进程环境或真实密钥；文档仅输出/tmp合成目录。
- 独立审查及错误保留修复复审：无剩余可操作发现。

## 未完成

截图完整动画消退、截图无限等待/Esc/保存/重触发/退出及子进程回收仍未实现。本单元不宣称解决截图问题。

## 补充回归与安装

validate_selection.py三个临时原生场景selection/selection-recovery/selection-lifecycle均通过，分别覆盖显式导入错误、重载后材料恢复、隐藏/展开不被迟到读取抢回。普通release通过且不含FOUNDATION_SMOKE。

普通SHA256 71ee6a819ffac40c65d64a74e7ba2c59bdab2c6f939b1fe1e999881418b1369e，已原子安装artifacts/zwen，备份zwen-before-selection-clear。未关闭/启动用户实例，完整退出重开生效。
