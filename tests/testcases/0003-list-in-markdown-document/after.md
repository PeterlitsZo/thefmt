**Files:**

- Modify: `crates/daec_config/src/lib.rs`
- Modify: `crates/daec_core/src/lib.rs`
- Modify: `crates/daec_infra/src/client/mod.rs`
- Modify: `bin/daec_cli/src/main.rs`
- [ ] 新增公开策略枚举，例如 `PrevTradeDaySnapshotPolicy`，至少包含：
  - `Disable`
  - `UsePrevTradeDayBeforeOpenAndClosure`
- [ ] 在 `ClientConfig` 中增加策略字段，并补充注释：
  - 说明这是初始化时必须显式指定的行为策略。
  - 说明若选择回退到上一交易日快照，则服务必须提前部署并长期稳定运行，才能在前一
    交易日下午完成固化。
  - 说明若策略开启而 `db_path` 为 `None`，初始化必须返回 `Error`。
- [ ] 在 `DataApiEcClient::new` 的注释里补充同样的行为约束，避免调用方误用。
- [ ] 在 `LowClientOptions` 中透传该策略，并在 `LowClient::new` 内做参数校验：
  - `Disable` 允许 `db_path=None`
  - `UsePrevTradeDayBeforeOpenAndClosure` 禁止 `db_path=None`
- [ ] 在 CLI 示例配置里显式填写该策略，避免编译报错，也让示例体现新配置项。
- [ ] 在初始化成功处补日志：
  - 当前策略值
  - `db_path` 是否启用
  - 是否允许 prev-trade-day 回退

--------------------------------------------------------------------------------

1. Some text, and code block below, with newline after code block

   ```yaml
   ---
   foo: bar
   ```

   1. Another
   2. List

1. Some text, and code block below, with newline after code block
   1. Another
   2. List

   ```yaml
   ---
   foo: bar
   ```

   1. Another
   2. List

--------------------------------------------------------------------------------

## v0.1.0 (2026-03-17)

### 新增

- 新增对话全生命周期管理能力，支持创建、更新、置顶、删除、批量删除、计数和列表查
  询。
- 新增消息管理与生成能力，支持消息创建、更新、列表查询，以及消息续写和重生成。
- 新增 AI 会话能力，支持流式响应、对话建议和跟进建议。
- 新增资讯选股能力，支持概览列表、最新概览和详情查询，并返回数据耗尽标识。
- 新增 AI 诊股能力，支持发起诊股、查询任务状态与结果、查询用户诊股历史、查询个股
  最新诊股并更新用户评分。
- 新增操作记录管理能力，支持列表、详情、更新、删除、批量删除和过滤查询。
- 新增行情与资讯流查询能力，支持列表与详情查询。
- 新增市场数据能力，支持股票列表、股票搜索和指数批量查询。
- 新增自选能力，支持添加、移除和列表查询。
- 新增运行与观测基础能力，提供健康检查、指标、请求日志、追踪日志和告警能力。

--------------------------------------------------------------------------------

## 如何使用

你可以按自己的工作方式来使用本仓库中的规范。常见方式如下：

1. 直接参考本文档。

   适合先快速了解整体规范的时候。你可以直接阅读 `README.md`，并在需要时继续查看
   `refs/` 下更细的参考文档。

2. 安装 `project-bootstrap` skill。

   - **让 AI 帮忙安装**：
     在你喜欢的 AI CLI 对话框中复制如下指令并回车：

     ```text
     现在你需要帮助用户安装 project-guide 提供的 skills。请严格参考如下步骤：

     - 询问用户在哪里克隆 `project-guide` 项目？例如 `~/Project/ft/project-guide/` 等。
     - 使用 `git@code.non-convex.com:hfrc/open/project-guide.git` 完成克隆。
     - 将项目的 `skills/project-bootstrap` 创建软链接到你的 skills 目录：
       - 如果你是 Codex，则是 `~/.codex/skills/`。
       - 如果你是 Claude Code，则是 `~/.claude/skills/`。
     - 告知你的用户已经完成安装。
     ```

   - **手动安装**：

     假设你的 Codex skills 目录是默认值 `~/.codex/skills`，那么可以执行：

     ```bash
     mkdir -p ~/.codex/skills
     ln -s /path/to/project-guide/skills/project-bootstrap \
       ~/.codex/skills/project-bootstrap
     ```

     如果你不想使用软链接，也可以直接复制目录（建议使用软链接，以使用 git pull
     来及时更新 skill）：

     ```bash
     mkdir -p ~/.codex/skills
     cp -R /path/to/project-guide/skills/project-bootstrap \
       ~/.codex/skills/project-bootstrap
     ```

     安装后，重启 Codex，使其重新加载 skills。

     重启后，可以在对话中直接点名使用这个 skill。例如：

     ```text
     请使用 project-bootstrap skill，帮我为一个 Rust 后端项目生成初始化方案。
     ```

     或者：

     ```text
     使用 project-bootstrap，为当前仓库补齐 AGENTS.md、CHANGELOG.md 和
     CONTRIBUTING.md 草案。
     ```

     如果你的目标比较明确，建议在提问时一并说明项目类型、是否是新仓库、希望直接
     生成哪些文件。这样 skill 会更容易给出可执行的初始化结果。

--------------------------------------------------------------------------------

- 项目概览。本节需要给出项目大体目的。包括项目的目的、项目想要解决的问题等。
- 项目结构。告知 AI 本项目是如何被组织的。这个能帮助 AI 快速定位到问题所在的文
  件。
- 让 AI 阅读 `CONTRIBUTING.md`，让它知晓如何贡献。
- 行为规范和开发提示。告知 AI 如何参与本项目。例如哪些行为是明确禁止的，开发前后
  应该执行什么。这里一些是所有项目通用的规则，一些则和项目本身有关系。对于一些操
  作有着严格的操作顺序时，则尽可能写明白点。
- 推送规范。建议在本节中告知如何编写 Git 提交消息、在提交前应该怎么做（例如，维
  护 `CHANGELOG.md`）。
- 交互提示。在本节中指出如何和用户交互。

--------------------------------------------------------------------------------

- 扩展 `/api/v1/system/info` 返回内容，新增基于 `shadow-rs` 的构建、Git、feature
  与前端构建信息，并同步更新接口文档。
- 抽离系统信息生成逻辑到 `src/build_info.rs`，复用到系统接口返回值中。
- 为可执行文件补充 `--version` 和 `build-info` 命令行输出。
- 为 `app01_prod`、`*******_app` 和 `*******_app_test` 显式补充 `message_center`
  配置，避免依赖默认值。
