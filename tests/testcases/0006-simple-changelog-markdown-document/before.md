# CHANGELOG

本文件记录项目的重要变更。

格式参考 Keep a Changelog，并结合项目实际情况使用中文标题。

## 未发布

### 变更

- 将格式化、lint、测试脚本入口统一切换为 `scripts/*.lua`。
- 收紧 `clippy` 默认检查规则，补充更严格的静态检查。
- 拆分 `commons_redis` 与 `commons_redis_lock_manager` 的内部模块结构，收敛
  Redis 相关公开类型命名。
- 将 `commons` 的重导出放置到子模块中。同时让多个类型的名称变得更短。

### 新增

- 新增 `commons_redis_lock_manager` crate，提供基于 `commons_redis::Pool`
  的 Redis 分布式锁能力，并由根 crate `commons` 统一重导出。

### 修复

- 增强 `commons_redis` 的 Sentinel 恢复能力，补齐 Sentinel 节点自身连接配置，
  支持将 Sentinel 节点与 Redis 数据节点的认证和连接参数分开设置。

### 文档

- 补充 `commons` 等使用示例和说明。
- 新增 RFC 0001 -- Redis Task Manager。

## v0.1.0 (2026-05-10)

### 新增

- 初始化 `commons-rs` workspace 骨架。
- 新增 `commons`、`commons_macros` 与 `commons_trace_fn`、`commons_redis` 四个
  crate。用于提供 `#[trace_fn(...)]` 宏和 Redis 连接。
