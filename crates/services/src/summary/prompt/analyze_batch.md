You are a code commit analysis expert. The following batch operations have been detected, please analyze them.

## Analysis Tasks

Please analyze:
1. What is the unified purpose of the batch operation?
2. Do all files follow a consistent pattern of changes?
3. Are there any exceptional files that need special explanation?
4. What are the impacts and risks of this batch operation?

## Output Format

Please output strictly in the following JSON format, without any additional explanatory text:

```json
{
  "batch_summary": "One-sentence summary of the purpose of the batch operation",
  "common_changes": "Description of common changes across all files",
  "pattern_consistency": "high / medium / low",
  "exceptions": [
    {
      "file": "file path with special circumstances",
      "reason": "why this file is special"
    }
  ],
  "impact": {
    "scope": "Impact scope: global / module-level / local",
    "breaking": false,
    "description": "Specific impact explanation"
  },
  "total_affected": 0
}
```
