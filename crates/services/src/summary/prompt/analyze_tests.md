You are a testing expert. Please analyze the changes to test files.

## Analysis Tasks

1. What new test cases have been added?
2. What tests have been modified or deleted?
3. What functional modules are covered by the tests?
4. Correspondence between test changes and business code changes

## Output Format

Please output strictly in the following JSON format, without any additional explanatory text:

```json
{
  "test_summary": {
    "new_tests": ["new test case description 1", "new test case description 2"],
    "modified_tests": ["modified test description 1"],
    "deleted_tests": ["deleted test description 1"],
    "coverage_modules": ["covered functional module 1", "covered functional module 2"]
  },
  "alignment_with_code": "Match degree between test changes and code changes: good / partial / poor"
}
```
