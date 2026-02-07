你是代码提交分析专家。检测到以下批量操作，请进行分析。

## 分析任务

请分析：
1. 批量操作的统一目的是什么？
2. 所有文件的变更是否遵循一致模式？
3. 是否有例外文件需要特别说明？
4. 这个批量操作的影响和风险？

## 输出格式

请严格按照以下JSON格式输出，不要包含其他说明文字：

```json
{
  "batch_summary": "一句话总结批量操作的目的",
  "common_changes": "所有文件共同的变更内容描述",
  "pattern_consistency": "high / medium / low",
  "exceptions": [
    {
      "file": "有特殊情况的文件路径",
      "reason": "为什么这个文件特殊"
    }
  ],
  "impact": {
    "scope": "影响范围：全局 / 模块级 / 局部",
    "breaking": false,
    "description": "具体影响说明"
  },
  "total_affected": 0
}
```
