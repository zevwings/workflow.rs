# 定义与实现分离计划

将 trait 定义与具体实现拆分到不同 crate，实现依赖倒置与可替换实现。

---

## 依赖关系

```
client (定义层)     ←  trait、类型、错误
      ↑
infra (实现层)       ←  具体实现（reqwest、OpenAI 等）
      ↑
storage, services, app
```

---

## 优先级 1（高）

**目标**：外部 API 客户端 → `client` 定义 + `infra` 实现

| 模块 | 定义层 (client) | 实现层 (infra) |
|------|------------------|----------------|
| HTTP | `HttpClient` trait | reqwest 实现 |
| LLM | `LLMClient` trait | OpenAI 兼容实现 |
| GitHub | `GitHubClient` trait | GitHub API 实现 |
| Jira | `JiraClient` trait | Jira API 实现 |

**当前**：http、llm 为单 crate；GitHubClient、JiraClient 在 storage 内定义并实现。

---

## 优先级 2（中）

**目标**：配置仓储 → `domain` 定义 + `infra-config-*` 实现

**触发条件**：存在多种配置来源（文件、环境变量、远程等）

| 模块 | 定义层 (domain) | 实现层 (infra) |
|------|-----------------|----------------|
| 全局配置 | `GlobalConfigRepository` | `infra-config-fs` |
| 项目配置 | `RepoConfigRepository` | `infra-config-fs` / `infra-config-env` |

**当前**：仅文件系统实现，暂可不拆。

---

## 优先级 3（可选）

**触发条件**：有多实现需求或解耦测试依赖时

| 模块 | 定义层 | 实现层 |
|------|--------|--------|
| Prompt Backend | `Backend` trait | crossterm / mock |
| 模板引擎 | `TemplateEngine` trait | handlebars / 其他 |

---

## 相关文档

- [HTTP 定义抽离指南](./http-definition-extraction.md)
- [架构设计](./architecture.md)

---

**最后更新**: 2025-02-13
