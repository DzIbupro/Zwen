# 技术选型清理记录

日期：2026-10-29。用户此前明确要求选型后删除未采用的依赖及下载/编译产物。

已确认没有本次样例仍从 build 目录运行，随后删除仅属于此次测试的：

| 目录（technology-spike-2026-10-29 下） | 删除前已分配空间 |
|---|---:|
| build/ | 1,624,164 KiB |
| cargo-cache/ | 258,440 KiB |
| web/vendor/ | 9,204 KiB |

合计 **1,847.5 MiB，约 1.80 GiB**；删除后已检查三个目录均不存在。按 du 的已分配块统计，受文件系统快照/共享块影响，不承诺可用空间精确增加同样数字。精简源码、锁文件、测量结果保留在仓库，原始快照/日志仍留在本机，不保留测试二进制。

系统包使用 `dnf remove --assumeno --no-autoremove` 预览，确认只移除本次新增的 gtk4-devel、webkitgtk6.0-devel、webkitgtk6.0、javascriptcoregtk6.0-devel、javascriptcoregtk6.0，预估释放 145 MiB。由于 sudo 需要密码，由用户执行卸载；用户回复“已清理”后，rpm 查询确认五个包均已不存在，rust、cargo、webkit2gtk4.1-devel 仍在。

保留选中架构的构建依赖与共享系统依赖；未执行全局 autoremove，未回滚 DNF 事务23，也未调整此前事务涉及的系统服务版本。将来复现 GTK4/WebKit6 候选需重新安装对应开发包；Cargo/vendor 缓存也需重下，这符合用户优先节省存储的选择。
