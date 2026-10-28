# 开发说明

## 环境

优先在 Fedora 45 + Niri 26.04 真实 Wayland 会话验收。Python 最低 3.11；本机实测为 Python 3.15。
PyGObject、GTK4、libsoup 3 使用发行版系统包，不将 GTK 本身写成 pip 依赖。
原型环境已有 python3-gobject-base、gtk4 和 libsoup 3 typelib；layer-shell 非首版必需。

```bash
python3 scripts/check_environment.py
python3 -m venv --system-site-packages .venv
.venv/bin/python -m pip install -e .
```

虚拟环境需要 `--system-site-packages` 才能读取发行版提供的 GI 绑定。
如果系统尚未提供 venv/pip/setuptools，先通过 Fedora 软件包管理器补齐；不要使用 sudo pip。
可直接通过 `PYTHONPATH=src` 导入源码，不必先完成打包安装。

## 基础检查

```bash
python3 -m compileall -q src scripts
PYTHONPATH=src python3 -c 'import study_assistant; print(study_assistant.__version__)'
```

当前尚无产品行为单元测试。首个业务任务必须建立测试，随后使用 `python3 -m unittest discover -s tests -v`。
GUI、PRIMARY、焦点、截图、输入法和缩放必须另做真实桌面验收，不能把无显示环境的测试称为兼容性通过。

## 约定

- UI 操作在 GTK 主线程；网络使用异步 API；SQLite 工作线程独占其连接。
- 草稿先保存；失败不丢内容；停止后拒绝迟到流片段。
- 截图先监听事件再发请求，单请求独立路径；命令返回0不是完成证据。
- 不将截图、聊天数据库、密钥、原始机器日志提交到 Git。
- 不自动改 Niri 配置；快捷键安装先检查主文件和 include，保留退出快捷键。
- 低内存目标须与刷新性能、图片尺寸和历史规模一起验收。
- 添加模型 Provider 前确认服务协议和图片能力，只使用用户明确配置的密钥。

暂不选择开源许可证；需要发布或接受贡献时由仓库所有者确定。
