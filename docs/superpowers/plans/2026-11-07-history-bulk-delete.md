# 历史批量删除与全部清空

用户已明确要求，沿单条删除边界完成。

1. Journal.delete_history(Some(ids))/None在一个Immediate事务验证所有目标并删除，前者只删明确ID，后者包含全部分页。空选择/非法ID拒绝；重复/缺失幂等；任一生成中/存储故障使整批回滚。单条包装复用同一逻辑。只处理turns，不删除收藏Markdown或草稿。
2. 新delete_chat_history(scope) IPC使用selected{requestIds}/all枚举；沿用chat mutex和active/unsaved保护；成功后仅失效已删除缓存，返回实际删除数。清空在确认请求被处理时针对事务快照；之后新创建的问答保留。
3. HistoryController checked跨页保留，全选本页和取消勾选；批量操作使用确认时保存的ID副本，成功清理匹配详情/勾选/当前App引用并回第一页，失败保留选择和详情。全部删除独立按钮及明确跨分页确认，不拿当前20条循环冒充全部。
4. 前端、临时SQLite、原生合成25+条跨页取消/批删/清空/缓存及草稿收藏保留回归，独立审查，普通版原子安装和续接记录。不请求模型或操作真实历史。
