# Zwen 接手开发审计包（面向审查 AI）

生成时间：2026-11-02（Asia/Shanghai）。作者：上一任执行 AI。
**用途：让另一个 AI 独立复核本人在接手后所做的全部开发，验证证据真伪，并指出需要修改的细节。**
本文自包含：可直接发送给没有任何上下文的其他 AI。本文不是宣传材料，包含已知缺陷、测试驱动自身的问题、以及无法在当前环境完成的验证项。

> 阅读顺序：先读 §1 的"如何验证（命令清单）"并实际执行；再按 §3 逐单元核对代码与证据；最后按 §5 的待审清单给出结论。**请不要只看本文的结论——本文所有数字都给出了可复现的命令。**

---

## 1. 基线与如何验证

### 1.1 环境

| 项 | 值 |
| --- | --- |
| 仓库 | `/home/dzibupro/Zwen/Zwen`（两层 Zwen） |
| 远端 | https://github.com/DzIbupro/Zwen （私有，草稿 PR #1） |
| 分支 | `feat/desktop-foundation`（未合并 main，按约定保持草稿） |
| 接手基线 | `9a3efc9f059ca51fd571d7a087ce2c7659459800`（交接文档 AI-HANDOFF.md 记录值） |
| 当前 HEAD | 审计文档提交后的 `HEAD`（核实时执行 `git rev-parse HEAD`；功能 HEAD 为 `7990699`，之后仅有接线修复与本文档） |
| 系统 | Fedora 45，Niri 26.04 / Wayland，gnome-keyring 可用 |

工具路径（不在默认 PATH）：

```bash
export PATH="/home/dzibupro/.cache/codex-runtimes/codex-primary-runtime/dependencies/bin/fallback:/home/dzibupro/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin:$PATH"
CARGO_HOME="$PWD/.cache/cargo"   # 离线依赖缓存
```

### 1.2 复现全部自动化验证

```bash
cd /home/dzibupro/Zwen/Zwen
git rev-parse HEAD                                   # 应为 7990699 或其后继

# Rust（core + transport + desktop 单元）
CARGO_HOME="$PWD/.cache/cargo" cargo test --offline
# 期望：passed 95, failed 0（core+transport；默认 members 不含 GUI 单元）
CARGO_HOME="$PWD/.cache/cargo" cargo test --offline -p zwen-desktop
# 期望：passed 7, failed 0, 1 ignored（含组织/密钥单元 + 命令一致性守卫；ignored 是真实钥匙环往返）

# 前端
pnpm test        # 期望 82 passed（14 个文件）
pnpm check       # 期望 0 errors 0 warnings
pnpm build

# 原生合成回归（9 个脚本，需先确认没有用户实例在跑：pgrep -x zwen）
CARGO_HOME="$PWD/.cache/cargo" cargo build --offline --release -p zwen-desktop --features desktop-smoke -j 2
python3 scripts/validate_desktop.py    "$PWD/target/release/zwen"
python3 scripts/validate_capsule.py    "$PWD/target/release/zwen"
python3 scripts/validate_selection.py  "$PWD/target/release/zwen"
python3 scripts/validate_provider.py   "$PWD/target/release/zwen"
python3 scripts/validate_chat.py       "$PWD/target/release/zwen"
python3 scripts/validate_chat_fault.py "$PWD/target/release/zwen"
python3 scripts/validate_chat_image.py "$PWD/target/release/zwen"
python3 scripts/validate_screenshot.py "$PWD/target/release/zwen"
python3 scripts/validate_knowledge.py  "$PWD/target/release/zwen"
```

**注意事项（踩过的坑，审查时也会遇到）**

1. 所有原生脚本会先 `pgrep -x zwen` 断言单实例空闲；**若用户实例在运行会直接失败**。不要 kill 用户进程。
2. `src-tauri/tests/*.js` 通过 `include_str!` 编入二进制。**改了驱动必须重新构建**，否则跑的是旧驱动（我因此误判过两次）。
3. 部分脚本会在 `~/Downloads/Zwen导出/` 写入导出文件（合成内容）；审查前可先 `ls`，审查后可删除。
4. `cargo fmt` 在本机未安装，代码风格为手写紧凑风格；不做格式门禁。

### 1.3 交付产物

| 产物 | 说明 |
| --- | --- |
| `artifacts/zwen` | 普通发布二进制（**必须不含** desktop-smoke） |
| `artifacts/validation/ordinary-binary.sha256` | 当前值 `190d8176ae2dd079d95f088a2e6a85eda21f18740c34ae1159161425942554cc`（2026-11-02 补注册 `notes_update_source` 后重建） |
| `artifacts/validation/native-suite.log` | 原生回归累积日志（含多轮追加段） |
| `artifacts/validation/*.json` | 各脚本的结构化结果 |

校验命令：`sha256sum artifacts/zwen`；`strings artifacts/zwen | grep -c FOUNDATION_SMOKE` 应为 `0`。

---

## 2. 接手后完成的工作总览（按提交顺序，含动机）

共 **24 个提交**（`git log --oneline 9a3efc9..HEAD` 可数）。按主题分组：

### 阶段 A：截图附件 + 视觉模型（用户明确要求的下一单元）

| 提交 | 内容 | 动机来源 |
| --- | --- | --- |
| `d1138db` | 截图附加胶囊草稿：唯一 request id/路径、事件+轮询、成功判定、取消/超时/迟到归属、PNG 边界、缩略图、删除/重截 | 交接文档 §10 |
| `592bfce` 等文档提交 | 证据与哈希 | 约定 |
| `2027846` | 视觉模型路由：可选 imageModel、官方 content 数组+裸 base64、有界负载、附件消费、发送闸门 | 交接文档 §10 后续 |
| `d8ebff5` | 模型密钥存入系统钥匙环（zbus/Secret Service），环境优先、钥匙环兜底 | 用户诉求"每次都要输密钥" |

### 阶段 B：用户真实使用驱动的缺陷修复

| 提交 | 内容 | 真实反馈原文/现象 |
| --- | --- | --- |
| `21403d5` | **截图期间隐藏助手窗口** | "页面会错乱并且会有一些页面处于半透明状态，会遮挡需要截图的地方" |
| `64bd16e` | **先切换 surface 再 show**（消除胶囊残影） | 用户截图显示胶囊尺寸窗口里出现完整窗口标题栏残影并闪烁 |
| `b53d74d` | **等待中重触发=重新开始截图** | "第一次点击快捷键之后能启用截图…第二次点击快捷键就会唤起胶囊但不会截图…循环往复" |
| `a57086d` | **发送即清空输入，零字失败回填** | "模型还在思考正在生成回答的时候…输入框中仍然有上一次提问的问题" |
| `dd0dfe2` | 零字失败**保留截图与草稿**供一键重试 | 用户真实 429/1305 后需要重新截图重打 |
| `8ae45bb` | 记录首次真实视觉调用结果（429/1305） | 用户实测 |

### 阶段 C：知识库（路线图阶段 3）

| 提交 | 内容 |
| --- | --- |
| `67aa343` | Markdown 知识库：`notes/<id>.md` 权威 + `notes.sqlite3` 可重建索引、原子写、孤儿采纳、中文子串搜索、收藏/查看/删除/重建 |
| `eb55a32` | 来源回看（`request:` 链接→历史详情）+ 应用内编辑（标题/正文） |
| `a51560f` | 来源元数据（标题/URL/页码）+ 单篇/全部导出到 `~/Downloads/Zwen导出/` + 外链走系统浏览器 |

### 阶段 D：AI 整理（阶段 4）

| 提交 | 内容 |
| --- | --- |
| `cf67acb` | 数据模型+管线：`organize.sqlite3`（topics/memberships/batches/relations/meta）、批次 changeset 撤销、范围强校验、提示词构建、模型输出解析校验、单任务互斥 |
| `5301ef0` | 整理 UI：勾选 1–8 篇 → **每次**确认视图明示模型与范围 → 结果 → 一键撤销；主题条 |
| `388d3c9` | 主题治理（改名/栏目/合并/拆分/锁定/删除）+ 关系（**两侧必须都有该笔记作为证据**）+ 自动增量开关 |

### 阶段 E：图谱与浮条（阶段 5、6）

| 提交 | 内容 |
| --- | --- |
| `bf9f0f5` | 手写 SVG 知识网络：节点=主题（大小=归属数）、边=关系、缩放/拖动/筛选/按需展开、点笔记跳详情 |
| `7990699` | 自动划词浮条：设置开关（持久化+启动恢复）+ `wl-paste --primary --watch` → 候选胶囊，不自动发送 |

---

## 3. 逐单元代码地图（审查入口）

### 3.1 截图链路

- `crates/core/src/screenshot.rs`：`capture_with()` 状态机。**审查重点**：成功判定（命令成功+文件稳定 250ms+PNG 解码）、取消/超时清理、`cleanup_incoming` 只删 32-hex 文件名、事件解析只认字符串 path。
- `crates/core/src/attachment.rs`：`import/replace/remove/thumbnail/model_payload`。**重点**：单张 24MiB/10MP/8192、请求负载≤4MiB/6000px 的收敛逻辑、文件名与权限（0600/0700）、`replace` 事务内替换、`note_hash` 无关。
- `src-tauri/src/attachment.rs`：`start/run`。**重点**：`hide_if_current` 在 `capture` 之前；结束时 `activate_if_current(同 intent)`；等待中重触发=丢弃旧等待并新开请求。
- `src-tauri/src/windows.rs`：`activate_if_current` 中 **surface-changed 早于 show**、120ms 等待、show 前重查 intent。

### 3.2 视觉模型

- `crates/core/src/provider.rs`：`image_model` 可选字段；**留空时序列化与旧格式逐字节一致**（对话绑定兼容），`with_model()` 只换模型名。
- `crates/core/src/completion.rs`：`Message` 自定义 Serialize——无图片时输出字符串 content，有图片时输出官方 `[{type:image_url},{type:text}]` 数组。
- `crates/transport/src/lib.rs`：图片数量≤8、base64 字符集与 8MiB 上限，**非法负载在发请求前拒绝**。
- `src-tauri/src/chat.rs`：视觉路由；**附件消费时机是收到第一个回答字**（回调里删附件）。

### 3.3 密钥钥匙环

- `src-tauri/src/secret.rs`：zbus blocking 客户端；属性 `{xdg:schema: io.github.dzibupro.zwen, zwen-role: model-key}`；plain session；`GetSecret` 结构第 4 字段是**字符串** content_type（首版误用 au，真实测试抓出）。
- `src-tauri/src/provider.rs`：返回 `keySource`（environment/keyring/null），**不回传密钥**。

### 3.4 知识库

- `crates/core/src/notes.rs`：front matter 解析/写入（id/created/source/model/title/attachments/request/source_*）、`create/update/update_source/delete/rebuild_index/list`。**审查重点**：`rebuild_index` 只采纳不删孤儿；删除笔记时清理文件；`valid_source_url` 只允许 http(s)；搜索对 `%_\` 转义。
- `src-tauri/src/notes.rs`：命令层 + 导出 + `open_external`（仅 http(s)，xdg-open 分离进程）。

### 3.5 AI 整理

- `crates/core/src/organize.rs`：`apply_batch`（scope 校验/名称校验/上限 8/16/32/changeset）、`undo_batch`（只删本批 AI 行与空 AI 主题）、治理方法、`add_relation`（两侧证据校验）、`auto_organize` 开关。
- `src-tauri/src/organize_job.rs`：`build_prompt`（每批≤8 篇、正文 4000 字截断、总长 256KiB）、`parse_model_output`（剥围栏、同笔记合并、丢未知/非法）、`run_organize_job`（单任务互斥、失败零修改）。

### 3.6 图谱与浮条

- `frontend/lib/graph.ts`：纯函数布局（圆环、边过滤、幂等展开）+4 测试。
- `frontend/KnowledgeGraph.svelte`：SVG、缩放/拖动/筛选；无障碍角色。
- `src-tauri/src/selection.rs`：`candidate_set_enabled/get_enabled`（`settings.sqlite3` schema1）、`start_candidate_watcher`（spawn `wl-paste --primary --watch <self> candidate-callback`）、启动恢复。

---

## 4. 证据及其边界（**最重要的一节**）

### 4.1 有合成证据的

| 验证 | 覆盖 | 证据位置 |
| --- | --- | --- |
| Rust 95 + 7 项 | core/transport 95（截图状态机 7、附件 8、provider 13、transport 9、notes 12、organize 5、其余既有）+ desktop 7（整理解析 6、密钥/接线守卫）| `cargo test` 与 `cargo test -p zwen-desktop` |
| 前端 82 项 | 附件/知识/图谱/链接/聊天/草稿/选区等控制器 | `pnpm test` |
| 原生 9 脚本 | 见 §1.2 | `artifacts/validation/` |
| knowledge 脚本 3 次连跑 | 收藏→搜索→编辑→来源→整理→撤销→治理→自动开关→导出→候选→图谱 | `native-suite.log` |
| 真实钥匙环往返 | 存合成值→读回→删除→确认空 | `docs/validation/2026-11-01-keyring.md`（`#[ignore]` 测试，手动跑） |

### 4.2 **没有证据、不可宣称的**

1. **真实 Niri 交互截图**：`niri msg action screenshot` 只在 2026-10-29 原型里由**上一任**验证过；本轮只用假 niri 合成验证。真实选区期间的隐藏行为由**用户口头确认**（"确实可以使用并收到完整回答"），但没有结构化验收记录。
2. **真实智谱视觉调用未成功**：唯一一次真实调用返回 429/1305（平台过载）。因此 4MiB/6000px 负载限值、裸 base64 兼容性、流式图片行为、错误码路径全部**未经真实服务确认**。
3. **真实 AI 整理未跑**：合成服务返回固定 JSON；真实模型的主题质量、拒答、超长输出行为未知。
4. **真实鼠标划词浮条**：只验证了开关持久化与 fake `wl-paste` 能触发；真实跨应用 PRIMARY 矩阵、浮条位置（当前是顶部锚点，不跟鼠标）未验收。
5. **资源验收**：只有 chat 合成负载一次采样（PSS 259.86 / RSS 512.02 MiB，进程树，50ms 采样），**不包含**图谱/多图/长时间；不等于成品达标。
6. **长时与多屏**：30 分钟/8 小时、100 次唤出、双屏 100%/150%、中文输入法矩阵、打包安装全部未做。

### 4.3 我自己的失误（供审查者参考，不隐瞒）

1. **文档数字笔误**：曾把 9 个回归脚本写成"10/10"、前端 82 写成"84"。已在 `docs/development-progress.md` 追加勘误段。
2. **测试驱动多次误报"产品缺陷"**：
   - `include_str!` 驱动改动后未重建二进制，导致跑旧驱动；
   - 驱动用 `.settings button[type=button]` 依赖 DOM 顺序，被新增按钮抢位（已改为按文字匹配）；
   - 驱动绕过 UI 直接 `invoke('save_provider')`，导致界面仍显示"未配置模型"（已改为走"重新读取"）；
   - `wait()` 参数顺序写错（label 传进 limit 位）；
   - 断言视图状态错误（从历史跳回后知识面板保留详情是**设计行为**，驱动却假设在列表）。
   - **教训**：断言前先确认视图/时序，测试失败先怀疑驱动。
3. **真实缺陷（已修）**：`notes_update_source` 在 build.rs 声明、权限已给，却漏注册进 `invoke_handler`（前端未用到该命令，故未被使用暴露）；已补注册，并新增 `wiring_tests::commands_permissions_and_handler_agree` 三方一致性守卫——**已实测：临时移除注册时该测试会失败并指出具体命令名**。`TEMP_COUNTER` 声明为 `const` 导致 ID 计数器每个使用点都是新原子；`organize_delete_topic` 初版用哨兵合并实现被目标缺失阻断；`restore_candidate_watcher_if_enabled` 调用早于 `app.manage(Storage)` 导致启动 panic；relations/meta 表在建库 SQL 里重复声明。
4. **未做代码格式门禁**：`cargo fmt` 不可用，代码为紧凑手写风格。

---

## 5. 请审查者重点核查的清单（按优先级）

### P0：安全性 / 数据正确性

1. **密钥永不外泄**：grep 所有 IPC 返回结构，确认没有任何字段能返回密钥值；确认日志（`eprintln`/错误 message）不含密钥；确认 `provider.sqlite3` 不含密钥。可参考 `secret.rs` 的 `ApiKey` Debug 脱敏。
2. **附件消费时机**：`src-tauri/src/chat.rs` 的 `run()` 回调里删附件——确认"零字失败"路径确实**不**删除附件，且前端 `settlePendingSend` 不会误清用户新输入。
3. **撤销语义**：`undo_batch` 只能删除本批 `batch_id` 的行；手工行（`added_by='manual'`）绝不能被删。构造一个含手工归属的库验证。
4. **范围校验**：`apply_batch` 必须拒绝 scope 外笔记；`add_relation` 必须要求证据笔记同时属于两侧主题。
5. **路径穿越**：`valid_id`/`sanitize_filename` 是否足以阻止 `../` 与绝对路径；`cleanup_incoming` 是否只删自己命名的文件。
6. **外链**：`open_external` 是否只允许 http(s)；渲染区链接是否有 `javascript:` 之类绕过。

### P1：用户可见行为一致性

6b. **命令三方一致性**：运行 `cargo test -p zwen-desktop wiring`；任何新增命令必须同时出现在 build.rs、capabilities 与 invoke_handler 三处，否则该测试失败。**审查时请把这条作为新增命令的强制门禁。**

7. 截图：等待期间再按快捷键是否**每次都重新截图**（用户报告的奇偶交替是否真的消失）——需要真实 Niri 验证。
8. 截图等待期间窗口是否隐藏；胶囊出现时是否还有残影（用户报告的现象）——需要真实环境确认。
9. 知识库：外部编辑 `.md` 后"重建索引"是否采纳而非删除；删除笔记是否同时删文件。
10. 整理：确认视图是否**每次**都显示模型名与篇数（需求要求"初次启用明确模型和范围"，实现为每次确认，是否可接受需用户判断）。
11. 图谱：筛选后边是否同步过滤；展开是否幂等（有单测，但可手验）。

### P2：资源与稳定性

12. 全进程树 PSS/RSS（含 WebKit 子进程）在以下场景：空闲、20 条长回答、图谱全展开、3 张截图附件。目标"约 500MB 以内或接近"。
13. 空闲 CPU（60 秒窗口）是否≤单核 1%。
14. 进程被 kill 后重启：草稿/聊天/笔记/整理结果是否一致恢复；未保存回答是否按设计保留/标记中断。

### P3：代码质量与可维护性

15. `parse_model_output` 的容错是否过宽（会静默丢弃非法主题而非报错）——是否符合"不能静默丢关键材料"的约定？请给出判断。
16. 各 `spawn_blocking` 中是否有可能长时间持锁（尤其 `windows::Controller` 与 `Storage` 的 Mutex）。
17. 新增的 12 个 organize 命令权限是否最小化；`allow-open-external` 是否被滥用。
18. 前端 `App.svelte` 已较臃肿（约 190 行单文件、多个控制器），是否需要拆分。

---

## 6. 建议的审查产出格式

请按以下结构回执，便于逐条落实：

```
结论：通过 / 有条件通过 / 不通过
P0 问题：
  - [文件:行] 问题描述 → 复现步骤 → 建议修法
P1 问题：…
P2 实测数据：…（附命令与输出）
P3 建议：…
与我（审查者）无法验证的项：…
```

---

## 7. 快速定位命令

```bash
# 看某单元完整改动
git show --stat <commit>
# 看截图状态机
sed -n '1,200p' crates/core/src/screenshot.rs
# 看整理解析与校验
sed -n '1,140p' src-tauri/src/organize_job.rs
# 看密钥是否可能泄漏
grep -rn 'expose()\|read_key()' src-tauri/src crates/ | grep -v test
# 看命令与权限是否一一对应
grep -o '"[a-z_]*"' src-tauri/build.rs | sort > /tmp/cmds.txt
grep -o 'allow-[a-z-]*' src-tauri/capabilities/main.json | sort > /tmp/perms.txt
# 手工对照两份清单
```

---

## 8. 相关文档索引

| 文档 | 内容 |
| --- | --- |
| `docs/AI-HANDOFF.md` | 接手时的原始交接（历史基线，勿当现状） |
| `docs/requirements.md` | 产品范围（用户确认的需求） |
| `docs/design.md` | 架构与各阶段落实记录（含 2026-11-01/02 段落） |
| `docs/development-progress.md` | 逐单元进度、证据、勘误、下一步 |
| `docs/validation/2026-10-31-screenshot.md` | 截图接口核对与合成证据 |
| `docs/validation/2026-10-31-vision.md` | 视觉协议核对、合成证据、真实 1305 记录 |
| `docs/validation/2026-11-01-keyring.md` | 钥匙环实现与真实往返测试 |
| `docs/validation/2026-11-01-knowledge.md` | 知识库/整理/图谱/候选各段验证 |
| `docs/providers/zhipu.md` | 智谱接入记录与真实调用状态 |

---

## 9. 一句话现状

**代码层面：需求文档中的阶段 0–6 均已实现并通过本地合成验证（Rust 95+7 / 前端 82 / 原生 9 脚本全绿）。**
**真实环境层面：截图与视觉模型链路已由用户确认可用，但结构化真实验收（多轮/停止/图片成功路径/整理主题质量/跨应用划词/资源与长时）尚未完成，不能视为发行就绪。**
