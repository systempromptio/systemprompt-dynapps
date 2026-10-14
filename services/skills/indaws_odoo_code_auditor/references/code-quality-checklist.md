# Code Quality Analysis Checklist

Focus on maintainability, duplication, and code health.

## Code Duplication
- Search for repeated code blocks across files
- Identify similar functions that could be unified
- Check for copy-pasted code with minor variations
- Recommend creating shared utilities/mixins

## Code Complexity
- Identify overly complex methods (>50 lines, deeply nested)
- Look for god classes/models with too many responsibilities
- Check cyclomatic complexity (excessive if/else chains)
- Flag methods with >5 parameters

## Code Maintainability
- Check for magic numbers/strings
- Verify proper error handling
- Look for TODO/FIXME comments
- Assess code readability

## Documentation Quality
- Check for docstrings in models and methods
- Verify README.md existence and quality
- Look for inline comments explaining complex logic
- Check if manifest has proper description

## Code Organization
- Proper separation of concerns
- Logical file structure
- Appropriate use of inheritance vs composition
- Module dependencies (tight coupling issues)

## Test Honesty (no fake coverage)
- **The test name must match exactly what it asserts.** A test
  `test_..._dotted_path` that uses a field without a dot (`email`) is a duplicate
  flat-field test in disguise: it leaves the named case uncovered while making it
  *look* covered. FAIL — flag it.
- **Coverage % is not proof.** A high-coverage suite can still miss the branch
  its test claims to hit. Verify the test actually exercises the asserted path
  (the specific input/branch/principal), not just that it runs green.
- **Assert behavior, not absence of error.** A test that only checks "no
  exception" proves little; assert the concrete result/side effect.
