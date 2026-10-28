# 测试入口

当前Python基线使用标准库unittest；最终测试工具随技术选型确定。先执行[选型验证](../docs/decisions/technology-evaluation.md)，业务测试随后加入。
核心案例：草稿重启恢复、迟到截图不附加、PRIMARY读取失败不使用旧剪贴板、停止后拒绝流片段。
运行：`PYTHONPATH=src python3 -m unittest discover -s tests -v`。
空测试目录不构成“测试通过”的证据。
