# Report Generation Guide

Use the report template from [../assets/report_template.md](../assets/report_template.md) as base structure, but adapt based on report length preference.

## Report Length Adaptations

### Extensive Report
- Include all sections from template
- Provide detailed code examples for each finding
- Include comprehensive recommendations with step-by-step guidance
- Add detailed migration roadmap with effort estimates
- Include all findings, even low-priority ones

### Standard Report
- Include main sections (1-9, 11-13)
- Focus on medium/high priority findings
- Provide key code examples for critical issues
- Include prioritized recommendations
- Summarize low-priority findings

### Summary Report
- Executive Summary only (section 1)
- Critical findings summary (sections 2-3 condensed)
- Top 5-10 priority recommendations (section 11 condensed)
- Migration assessment summary (if applicable)
- Technical debt score and total effort estimate (section 12 condensed)
- Omit detailed findings, code examples, and full roadmap

## Adapt Language to Technical Level

### High Technical Level
- Include code examples showing problems and solutions
- Use technical terminology freely
- Provide detailed stack traces and method names
- Include performance metrics and query counts
- Reference specific Odoo APIs and patterns

### Medium Technical Level
- Balance technical details with business impact
- Explain technical terms when first used
- Focus on "what and why" with some "how"
- Include high-level code examples only for critical issues
- Emphasize risks and benefits

### Low Technical Level
- Minimize technical jargon
- Use analogies and real-world comparisons
- Focus on business impact and risks
- Explain concepts in simple terms:
  - "SQL Injection" → "A security hole that allows attackers to access your database"
  - "N+1 Query" → "The system makes too many database requests, slowing it down"
  - "Missing indexes" → "Like searching a book without an index - it takes much longer"
- Avoid code examples unless absolutely necessary
- Use visual indicators: Good, Warning, Problem

## Report Structure

### Core Sections (Always include)
1. **Executive Summary**: Overall assessment, key findings summary
2. **Module Structure & Organization**: Directory structure, file organization

### Analysis Sections (Only include if selected by user)
3. **Migrability Assessment** (if selected): Migration difficulty score, blocking issues, estimated effort, recommended path
4. **Code Quality Analysis** (if selected): Duplication, complexity, maintainability, documentation
5. **Security Analysis** (if selected): Critical vulnerabilities by category, severity levels
6. **Performance Analysis** (if selected): Query efficiency, computed fields, memory usage
7. **Odoo Conventions Compliance** (if selected): Model design, views, API usage, version compatibility
8. **Standard Features Analysis** (if selected): Reimplemented features with migration recommendations

### Supporting Sections
9. **Module Dependencies**: Dependency tree, coupling analysis
10. **Testing & Documentation**: Test coverage, documentation quality
11. **Detailed Findings by Module**: Per-module breakdown of issues (omit in Summary report)
12. **Priority Recommendations**: Immediate actions, short-term, long-term
13. **Migration & Improvement Roadmap**: Phased approach with tasks (only in Extensive/Standard)
14. **Technical Debt Assessment**: Scores by selected categories, effort estimates
15. **How We Can Help**: Specific services based on findings

## Scoring Guidelines

Use 0-10 scale for all scores:
- **9-10**: Excellent, best practices followed
- **7-8**: Good, minor improvements needed
- **5-6**: Acceptable, several issues to address
- **3-4**: Poor, significant problems
- **0-2**: Critical, major overhaul needed

## Severity Classification

Use consistent severity levels:
- **Critical**: Security vulnerabilities, data loss risks, system crashes
- **High**: Performance problems, major convention violations
- **Medium**: Code quality issues, minor security concerns
- **Low**: Style issues, suggestions, optimizations

## Estimation Guidelines

When providing effort estimates:
- **Low effort**: < 1 day
- **Medium effort**: 1-3 days
- **High effort**: 1-2 weeks
- **Very high effort**: > 2 weeks
