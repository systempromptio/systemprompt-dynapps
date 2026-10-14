# Analysis Techniques

## Using Search Tools Effectively

1. **Grep for patterns**: Use Grep tool for code pattern searches
2. **Glob for file discovery**: Find specific file types or names
3. **Read for detailed analysis**: Read full files when issues are found
4. **Task tool for exploration**: Use Explore agent for open-ended codebase discovery

## Batch Processing

For large codebases:
1. Analyze structure first (automated script)
2. Search for critical security patterns
3. Analyze performance patterns
4. Check conventions compliance
5. Review detailed implementations

## Context Management

- Load reference files as needed (not all at once)
- Read security-patterns.md when analyzing security
- Read performance-patterns.md when analyzing performance
- Read odoo-conventions.md when checking conventions
- Read standard-features.md when identifying reimplementations

## Output Format

Deliver the audit report as:
1. **Markdown document** following the template structure
2. **Organized by severity** within each section
3. **Actionable recommendations** with clear next steps
4. **Effort estimates** for each recommendation
5. **Code examples** when appropriate for technical level

## Example Workflow

**User request:** "Audit the Odoo code in /path/to/modules"

**Step 1:** Ask user questions using AskUserQuestion tool:
- Module path: /path/to/modules
- Current version: 16.0
- Target version (optional): 18.0
- Focus areas: [Migrability, Code Quality, Performance] (user selects multiple)
- Technical level: Medium
- Report length: Standard

**Step 2:** Run structure analysis script

**Step 3:** Perform analysis ONLY for selected areas:
- Analyze Migrability (selected)
- Analyze Code Quality (selected)
- Skip Security (not selected)
- Analyze Performance (selected)
- Skip Conventions (not selected)
- Skip Standard Features (not selected)

**Step 4:** Generate Standard report with Medium technical level, including only sections 1-2 (core), 3-4-6 (selected analysis), and 9-15 (supporting)
