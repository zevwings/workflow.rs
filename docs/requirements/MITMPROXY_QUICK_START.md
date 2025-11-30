# mitmproxy 快速使用指南

## 🚀 快速开始

### 1. 安装 mitmproxy

```bash
# macOS
brew install mitmproxy

# 或使用 pip
pip3 install mitmproxy
```

### 2. 启动 mitmproxy 并开始记录

```bash
# 使用记录脚本启动 mitmproxy
mitmdump -p 8080 -s scripts/mitm_record.py
```

### 3. 配置代理环境变量

在另一个终端中：

```bash
export http_proxy=http://127.0.0.1:8080
export https_proxy=http://127.0.0.1:8080
```

或者使用项目的代理管理功能：

```bash
# 先手动设置系统代理为 127.0.0.1:8080
# 然后使用项目命令启用
workflow proxy on
```

### 4. 安装证书（HTTPS 支持）

1. 启动 mitmproxy 后，在浏览器中访问 `http://mitm.it`
2. 下载并安装对应平台的证书
3. macOS 安装步骤：
   - 双击下载的证书文件
   - 在"钥匙串访问"中找到 mitmproxy 证书
   - 双击证书，展开"信任"，选择"始终信任"

### 5. 执行请求

现在所有通过 `workflow` 命令发送的 HTTP 请求都会被 mitmproxy 捕获并记录。

例如：

```bash
workflow pr test-api 123
```

### 6. 查看记录的请求

```bash
# 列出所有请求
workflow mitm list

# 搜索特定请求
workflow mitm search "api.github.com"

# 查看请求详情
workflow mitm show request_20240101_120000.json

# 查看记录目录
workflow mitm dir
```

## 📝 使用示例

### 示例 1：记录所有请求

```bash
# 终端 1：启动 mitmproxy
mitmdump -p 8080 -s scripts/mitm_record.py

# 终端 2：设置代理并执行命令
export http_proxy=http://127.0.0.1:8080
export https_proxy=http://127.0.0.1:8080
workflow pr create
```

### 示例 2：只记录特定域名的请求

编辑 `scripts/mitm_record.py`，修改 `FILTER_DOMAINS`：

```python
FILTER_DOMAINS = ["api.github.com", "api.example.com"]
```

然后启动 mitmproxy：

```bash
mitmdump -p 8080 -s scripts/mitm_record.py
```

### 示例 3：查看最近的请求

```bash
# 列出所有请求（按时间倒序）
workflow mitm list

# 查看第一个请求的详情
workflow mitm show $(workflow mitm list | head -1)
```

## 🔧 高级配置

### 自定义记录目录

编辑 `scripts/mitm_record.py`，修改 `RECORD_DIR`：

```python
RECORD_DIR = Path("/path/to/your/records")
```

### 使用 mitmweb（Web 界面）

```bash
# 启动 Web 界面
mitmweb -p 8080

# 在浏览器中访问 http://127.0.0.1:8081
```

### 导出为 HAR 格式

```bash
# 导出为 HAR 格式（可以在浏览器开发者工具中打开）
mitmdump -p 8080 -w requests.har
```

## 📚 更多信息

详细文档请参考：[MITMPROXY_INTEGRATION_GUIDE.md](./MITMPROXY_INTEGRATION_GUIDE.md)


