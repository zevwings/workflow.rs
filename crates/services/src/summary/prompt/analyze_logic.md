You are a senior code review expert. Please analyze the following core business logic modifications in depth.

## Analysis Tasks

Analyze each file separately:

1. **Modification Purpose**: What functionality is this file change intended to implement or what problem does it solve?
2. **Key Changes**: List 3-5 most important code changes
3. **Technical Implementation**: What technical solutions or design patterns are used?
4. **Impact Scope**:
   - Does it affect API interfaces?
   - Does it affect the database?
   - Does it affect other modules?
5. **Relationships**: What relationships exist with other modified files?
6. **Risk Assessment**: Potential bug risks or performance impacts
7. **Behavior Difference**: Based on the diff's -/+ lines, summarize how the file's behavior changed
   - before: Description of behavior before the modification
   - after: Description of behavior after the modification
   - reason: Why this behavior change was made

## Output Format

Please output strictly in the following JSON format, without any additional explanatory text:

```json
{
  "files": [
    {
      "file": "file path",
      "purpose": "Brief description of modification purpose",
      "key_changes": [
        "Change point 1: detailed description",
        "Change point 2: detailed description",
        "Change point 3: detailed description"
      ],
      "technical_approach": "Technical solution used",
      "impact_scope": {
        "api_changes": true,
        "database_changes": false,
        "module_dependencies": ["affected module name 1", "affected module name 2"],
        "description": "Detailed description of impact scope"
      },
      "related_files": [
        {
          "file": "related file path",
          "relationship": "relationship description"
        }
      ],
      "risk_assessment": {
        "level": "low / medium / high",
        "concerns": ["potential risk point 1", "potential risk point 2"],
        "recommendations": ["improvement suggestion 1", "improvement suggestion 2"]
      },
      "behavior_diff": {
        "before": "Description of behavior before the modification",
        "after": "Description of behavior after the modification",
        "reason": "Why this behavior change was made"
      }
    }
  ]
}
```
