# 测试入口

使用 Python 标准库 unittest，业务测试从第一阶段任务开始加入。
核心案例：草稿重启恢复、迟到截图不附加、PRIMARY读取失败不使用旧剪贴板、停止后拒绝流片段。
运行：`PYTHONPATH=src python3 -m unittest discover -s tests -v`。
空测试目录不构成“测试通过”的证据。
