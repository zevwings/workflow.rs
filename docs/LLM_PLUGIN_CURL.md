# LLM API CURL 示例

本文档提供了不同 LLM 提供商的 API 调用示例，用于测试和验证配置。

## 📊 响应格式分析

所有 LLM 提供商都遵循 **OpenAI 兼容格式**，响应内容都从相同的路径提取：

### 统一提取路径

```json
{
  "choices": [
    {
      "message": {
        "content": "响应内容在这里"
      }
    }
  ]
}
```

**提取路径**：`choices[0].message.content`

### 各提供商验证

| 提供商 | 响应格式 | 提取路径 | 兼容性 |
|--------|---------|---------|--------|
| OpenAI | ✅ OpenAI 标准格式 | `choices[0].message.content` | ✅ 完全兼容 |
| DeepSeek | ✅ OpenAI 兼容格式 | `choices[0].message.content` | ✅ 完全兼容 |
| Cerebras PROXY | ✅ OpenAI 兼容格式 | `choices[0].message.content` | ✅ 完全兼容 |
| OpenAI PROXY | ✅ OpenAI 标准格式 | `choices[0].message.content` | ✅ 完全兼容 |
| Gemini (原生) | ⚠️ Gemini 原生格式 | `candidates[0].content.parts[0].text` | ❌ 不兼容，需自定义格式 |
| Gemini (兼容代理) | ✅ OpenAI 兼容格式 | `choices[0].message.content` | ✅ 完全兼容（通过代理） |

**结论**：
- **大多数提供商**都使用 OpenAI 兼容格式，可以使用统一的客户端实现
- **Gemini 原生 API** 使用不同的格式，需要通过自定义格式配置（`response_format = "custom"`，`content_path = "candidates[0].content.parts[0].text"`）
- **Gemini 通过 OpenAI 兼容代理**可以使用标准 OpenAI 格式

---

## OpenAI

### CURL
```
echo "=== OpenAI API 调用示例 ==="
curl -X POST "https://api.openai.com/v1/chat/completions" \
  -H "Authorization: Bearer YOUR_OPENAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-3.5-turbo",
    "messages": [
      {
        "role": "system",
        "content": "You are a helpful assistant."
      },
      {
        "role": "user",
        "content": "Hello, how are you?"
      }
    ],
    "max_tokens": 100,
    "temperature": 0.5
  }'

echo -e "\n\n"
```

### Response
```
{
  "id": "chatcmpl-CbGyRko0llk6UPZzjkz1YYy3GOyRj",
  "object": "chat.completion",
  "created": 1762999419,
  "model": "gpt-3.5-turbo-0125",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Hello! I'm just a computer program, so I don't have feelings, but I'm here and ready to assist you. How can I help you today?",
        "refusal": null,
        "annotations": []
      },
      "logprobs": null,
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 23,
    "completion_tokens": 33,
    "total_tokens": 56,
    "prompt_tokens_details": {
      "cached_tokens": 0,
      "audio_tokens": 0
    },
    "completion_tokens_details": {
      "reasoning_tokens": 0,
      "audio_tokens": 0,
      "accepted_prediction_tokens": 0,
      "rejected_prediction_tokens": 0
    }
  },
  "service_tier": "default",
  "system_fingerprint": null
}
```

---


## DeepSeek

### CURL
```
echo "=== DeepSeek API 调用示例 ==="
curl -X POST "https://api.deepseek.com/v1/chat/completions" \
  -H "Authorization: Bearer YOUR_DEEPSEEK_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "deepseek-chat",
    "messages": [
      {
        "role": "system",
        "content": "You are a helpful assistant."
      },
      {
        "role": "user",
        "content": "Hello, how are you?"
      }
    ],
    "max_tokens": 100,
    "temperature": 0.5
  }'

echo -e "\n\n"
```

### Response
```
{
  "id": "c0121bb1-9a82-4ea7-bc8b-3eae2b0956e0",
  "object": "chat.completion",
  "created": 1762999238,
  "model": "deepseek-chat",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Hello! I'm doing well, thank you for asking. How are you today? Is there anything I can help you with?"
      },
      "logprobs": null,
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 16,
    "completion_tokens": 26,
    "total_tokens": 42,
    "prompt_tokens_details": {
      "cached_tokens": 0
    },
    "prompt_cache_hit_tokens": 0,
    "prompt_cache_miss_tokens": 16
  },
  "system_fingerprint": "fp_ffc7281d48_prod0820_fp8_kvcache"
}
```

---



## Cerebras PROXY

### CURL
```
echo "=== 示例 2: 带 system message 的调用 ==="
curl -X POST "https://cerebras-proxy.brain.loocaa.com:1443/v1/chat/completions" \
  -H "Authorization: Bearer YOUR_PROXY_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen-3-235b-a22b-instruct-2507",
    "messages": [
      {
        "role": "system",
        "content": "You are an AI assistant that generates intelligent tab interfaces."
      },
      {
        "role": "user",
        "content": "Generate tabs for a travel planning focus."
      }
    ],
    "temperature": 0.7
  }'

echo -e "\n\n"
```

### Response
```
{
  "id": "chatcmpl-52804117-d100-4180-8c5d-5f4ee7b7c875",
  "choices": [
    {
      "finish_reason": "stop",
      "index": 0,
      "message": {
        "content": "Here’s a clean, intuitive tab interface designed for a travel planning application or website:\n\n---\n\n### **Travel Planning Dashboard – Navigation Tabs**\n\n```html\n<div class=\"travel-tabs\">\n  <button class=\"tab active\" data-tab=\"destinations\">🌍 Destinations</button>\n  <button class=\"tab\" data-tab=\"itinerary\">📅 Itinerary</button>\n  <button class=\"tab\" data-tab=\"flights\">✈️ Flights</button>\n  <button class=\"tab\" data-tab=\"lodging\">🏨 Lodging</button>\n  <button class=\"tab\" data-tab=\"activities\">🎯 Activities</button>\n  <button class=\"tab\" data-tab=\"budget\">💰 Budget</button>\n  <button class=\"tab\" data-tab=\"documents\">📄 Documents</button>\n</div>\n\n<!-- Tab Content Panels -->\n<div id=\"destinations\" class=\"tab-content active\">\n  <h2>Discover Your Next Destination</h2>\n  <p>Explore top-rated cities, hidden gems, and seasonal recommendations.</p>\n  <!-- Search bar, destination cards, etc. -->\n</div>\n\n<div id=\"itinerary\" class=\"tab-content\">\n  <h2>Build Your Daily Itinerary</h2>\n  <p>Plan day-by-day activities, set reminders, and sync with your calendar.</p>\n  <!-- Drag-and-drop planner, time slots, etc. -->\n</div>\n\n<div id=\"flights\" class=\"tab-content\">\n  <h2>Flight Search & Booking</h2>\n  <p>Compare prices, set fare alerts, and book flights with ease.</p>\n  <!-- Flight search form, deals, saved searches -->\n</div>\n\n<div id=\"lodging\" class=\"tab-content\">\n  <h2>Accommodation Options</h2>\n  <p>Find hotels, hostels, vacation rentals, and check availability.</p>\n  <!-- Filters for price, type, ratings, map view -->\n</div>\n\n<div id=\"activities\" class=\"tab-content\">\n  <h2>Things to Do</h2>\n  <p>Discover tours, attractions, local experiences, and book tickets.</p>\n  <!-- Activity cards, ratings, booking integration -->\n</div>\n\n<div id=\"budget\" class=\"tab-content\">\n  <h2>Travel Budget Tracker</h2>\n  <p>Set your budget, track expenses, and get cost-saving tips.</p>\n  <!-- Expense categories, graphs, currency converter -->\n</div>\n\n<div id=\"documents\" class=\"tab-content\">\n  <h2>Travel Documents</h2>\n  <p>Store passport info, visas, insurance, and emergency contacts securely.</p>\n  <!-- Upload, checklist, reminders for expiry dates -->\n</div>\n```\n\n---\n\n### Features & Rationale:\n- **Icons**: Visual cues improve recognition and user experience.\n- **Logical Flow**: Tabs follow a natural travel planning sequence.\n- **Mobile-Friendly**: Horizontal scroll or dropdown on small screens.\n- **Interactive**: JavaScript can toggle visibility of content panels.\n- **Customizable**: Add/remove tabs for specific needs (e.g., \"Packing List\", \"Weather\").\n\nWould you like a styled version (CSS) or interactive JavaScript functionality added?",
        "role": "assistant"
      }
    }
  ],
  "created": 1762995128,
  "model": "qwen-3-235b-a22b-instruct-2507",
  "system_fingerprint": "fp_cf9632e95879dbff8045",
  "object": "chat.completion",
  "usage": {
    "total_tokens": 700,
    "completion_tokens": 668,
    "prompt_tokens": 32
  },
  "time_info": {
    "queue_time": 0.000289052,
    "prompt_time": 0.003461869,
    "completion_time": 0.725624146,
    "total_time": 0.7304821014404297,
    "created": 1762995128.2197855
  }
}
```

## OpenAI PROXY

### CURL
```
echo "=== OpenAI PROXY API 调用示例 ==="
# 注意：使用 -k 选项跳过 SSL 证书验证（仅用于测试）
curl -k -X POST "https://openai-proxy.brain.loocaa.com:1443/v1/chat/completions" \
  -H "Authorization: Bearer YOUR_PROXY_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-3.5-turbo",
    "messages": [
      {
        "role": "system",
        "content": "You are a helpful assistant."
      },
      {
        "role": "user",
        "content": "Hello, how are you?"
      }
    ],
    "max_tokens": 100,
    "temperature": 0.5
  }'

echo -e "\n\n"
```

**注意**：此示例使用 `-k` 选项跳过 SSL 证书验证。如果服务器有有效的 SSL 证书，可以移除 `-k` 选项。

### Response
```
{
  "id": "chatcmpl-CbH30mPlvhOoCzlhsjm2M6xpBfdm9",
  "object": "chat.completion",
  "created": 1762999702,
  "model": "gpt-3.5-turbo-0125",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Hello! I'm just a computer program, so I don't have feelings, but I'm here and ready to help you. How can I assist you today?",
        "refusal": null,
        "annotations": []
      },
      "logprobs": null,
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 23,
    "completion_tokens": 33,
    "total_tokens": 56,
    "prompt_tokens_details": {
      "cached_tokens": 0,
      "audio_tokens": 0
    },
    "completion_tokens_details": {
      "reasoning_tokens": 0,
      "audio_tokens": 0,
      "accepted_prediction_tokens": 0,
      "rejected_prediction_tokens": 0
    }
  },
  "service_tier": "default",
  "system_fingerprint": null
}
```

---

## Gemini (原生 API)

**注意**：Gemini 原生 API 使用不同的响应格式，需要使用自定义格式配置。

### CURL
```
echo "=== Gemini API 调用示例 ==="
curl -X POST "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent?key=YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "contents": [
      {
        "parts": [
          {
            "text": "Hello, how are you?"
          }
        ]
      }
    ],
    "generationConfig": {
      "temperature": 0.5,
      "maxOutputTokens": 100
    }
  }'

echo -e "\n\n"
```

### Response
```
{
  "candidates": [
    {
      "content": {
        "parts": [
          {
            "text": "Hello! I'm doing well, thank you for asking. How are you today?"
          }
        ],
        "role": "model"
      },
      "finishReason": "STOP",
      "index": 0,
      "safetyRatings": [...]
    }
  ],
  "promptFeedback": {...}
}
```

**提取路径**：`candidates[0].content.parts[0].text`

**配置示例**（在 `llm.toml` 中）：
```toml
[[providers]]
name = "gemini-native"
enabled = true

[providers.config]
url = "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent?key=${GEMINI_API_KEY}"
api_key = "${GEMINI_API_KEY}"
response_format = "custom"

[providers.config.custom_format]
content_path = "candidates[0].content.parts[0].text"
```

---

## Gemini (OpenAI 兼容代理)

如果使用支持 OpenAI 兼容格式的 Gemini 代理服务，可以使用标准 OpenAI 格式。

### CURL
```
echo "=== Gemini OpenAI 兼容代理 API 调用示例 ==="
curl -X POST "https://gemini-proxy.example.com/v1/chat/completions" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gemini-pro",
    "messages": [
      {
        "role": "system",
        "content": "You are a helpful assistant."
      },
      {
        "role": "user",
        "content": "Hello, how are you?"
      }
    ],
    "max_tokens": 100,
    "temperature": 0.5
  }'

echo -e "\n\n"
```

### Response
```
{
  "id": "chatcmpl-gemini-xxx",
  "object": "chat.completion",
  "created": 1234567890,
  "model": "gemini-pro",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Hello! I'm doing well, thank you for asking. How are you today?"
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 23,
    "completion_tokens": 33,
    "total_tokens": 56
  }
}
```

**提取路径**：`choices[0].message.content`（与 OpenAI 标准格式一致）

**配置示例**（在 `llm.toml` 中）：
```toml
[[providers]]
name = "gemini-proxy"
enabled = true

[providers.config]
url = "https://gemini-proxy.example.com/v1/chat/completions"
api_key = "${GEMINI_PROXY_KEY}"
response_format = "openai"
```

---

