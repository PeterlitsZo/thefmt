**Files:**
* Modify: `crates/daec_config/src/lib.rs`
* Modify: `crates/daec_core/src/lib.rs`
* Modify: `crates/daec_infra/src/client/mod.rs`
* Modify: `bin/daec_cli/src/main.rs`
* [ ] 新增公开策略枚举，例如 `PrevTradeDaySnapshotPolicy`，至少包含：
  * `Disable`
  * `UsePrevTradeDayBeforeOpenAndClosure`
* [ ] 在 `ClientConfig` 中增加策略字段，并补充注释：
  * 说明这是初始化时必须显式指定的行为策略。
  * 说明若选择回退到上一交易日快照，则服务必须提前部署并长期稳定运行，才能在前一交易日下午完成固化。
  * 说明若策略开启而 `db_path` 为 `None`，初始化必须返回 `Error`。
* [ ] 在 `DataApiEcClient::new` 的注释里补充同样的行为约束，避免调用方误用。
* [ ] 在 `LowClientOptions` 中透传该策略，并在 `LowClient::new` 内做参数校验：
  * `Disable` 允许 `db_path=None`
  * `UsePrevTradeDayBeforeOpenAndClosure` 禁止 `db_path=None`
* [ ] 在 CLI 示例配置里显式填写该策略，避免编译报错，也让示例体现新配置项。
* [ ] 在初始化成功处补日志：
  * 当前策略值
  * `db_path` 是否启用
  * 是否允许 prev-trade-day 回退

---

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
