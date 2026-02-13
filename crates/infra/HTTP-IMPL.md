# infra HTTP 实现思路

client 定义 `HttpClient` trait，infra 用 reqwest 实现；无 http 独立 crate，无 global 单例。

---

## 依赖

`infra` 依赖 `client` + `reqwest` / `serde` / `base64` / `tracing`。

## 结构

```
infra/src/http/
├── client.rs     # ReqwestHttpClient impl HttpClient
├── request.rs    # Request builder → HttpRequest → execute()
├── response.rs   # reqwest::Response → client::HttpResponse
├── auth.rs       # Authorization → reqwest headers
├── multipart.rs
└── retry.rs
```

## 核心流程

1. `ReqwestHttpClient::new()` / `with_config()` 构造，持有 `reqwest::blocking::Client`
2. `get()` / `post()` 等返回 `Request<'_>` builder，链式累积参数
3. `Request::send()` 将 builder 转为 `HttpRequest`，调用 `execute(req)`
4. `execute()` 内：`HttpRequest` → reqwest → `reqwest::Response` → `HttpResponse`
5. `reqwest::Error` 转为 `HttpError`（提取 message，不持有 reqwest 类型）

## 使用

通过 DI 注入 `dyn client::HttpClient`，或直接 `ReqwestHttpClient::with_config(config)`。
