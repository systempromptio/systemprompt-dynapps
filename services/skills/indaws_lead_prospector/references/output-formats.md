# Output Formats — Lead Prospector

## Standard Lead Table

Output a markdown table sorted by quality score (highest first):

```markdown
| Company | Domain | Contact | Title | Email | Quality Score | Status |
|---------|--------|---------|-------|-------|---------------|--------|
| Empresa A | empresa-a.es | Juan Garcia | CEO | juan@empresa-a.es | 85 | PASS |
| Empresa B | empresa-b.com | Maria Lopez | CFO | maria@empresa-b.com | 65 | PASS |
| Empresa C | empresa-c.es | Pedro Ruiz | CTO | pedro@empresa-c.es | 45 | WARN |
```

Include summary statistics below the table:
- Total leads found
- PASS / WARN / FAIL counts
- Top industries represented
- Geographic distribution

## Enriched Lead Profile (for high-value targets)

```markdown
## [Company Name] — [Domain]

**Company Data:**
- Industry: [Industry]
- Employees: [Count]
- Revenue: [Range]
- Tech Stack: [Current tools/ERP]
- Recent News: [Summary]
- Location: [City, Region]

**Decision Makers:**

| Name | Title | Email | Work History Summary |
|------|-------|-------|---------------------|
| [Name] | [Title] | [Email] | [Brief history] |

**Quality Assessment:**
- Score: [X]/100 — [PASS/WARN/FAIL]
- Score breakdown: [List of checks and impacts]

**Prospecting Notes:**
- [Relevant buying signals]
- [Personalization hooks from work history or thought leadership]
- [Potential pain points based on industry/tech stack]
```

---

## Odoo CRM Import Field Mapping

See `references/shared/odoo-connection.md` for connection setup.

Use odoo-pilot to create leads with model `crm.lead`:

| Odoo field | Source | Example value |
|-----------|--------|---------------|
| `name` | Generated | "Lead: [Company Name] - [Contact Name]" |
| `partner_name` | Company name from Clay | "Empresa Example S.L." |
| `contact_name` | Contact name from Clay | "Juan Garcia" |
| `email_from` | Contact email from Clay | "juan@empresa.es" |
| `phone` | Contact phone from Clay (if available) | "+34 96 XXX XXXX" |
| `website` | Company domain from Clay | "https://www.empresa.es" |
| `function` | Contact title from Clay | "Director General" |
| `street` | Company address (if available) | "Calle Example 1" |
| `city` | Company city from Clay | "Valencia" |
| `country_id` | Spain | ID for Spain in Odoo |
| `description` | Enrichment summary | "Tech stack: [X]. Revenue: [Y]. Recent news: [Z]." |
| `source_id` | Fixed value | Source: "Clay Prospecting" |
| `medium_id` | Fixed value | Medium: "lead-prospector" |
| `campaign_id` | From user's campaign context | Campaign if specified |
| `tag_ids` | Quality gate result | Tags: "clay-lead", "PASS" or "WARN" |
| `priority` | Based on quality score | PASS >=80: "3" (very high), PASS >=60: "2" (high), WARN: "1" (medium) |

### Lead Creation Process

1. Confirm with user which leads to import (PASS only, PASS + WARN, or specific leads)
2. Check for duplicates in Odoo CRM before creating (search by domain and email)
3. Create leads via odoo-pilot with all available fields
4. Report created lead IDs and any duplicates found
5. Suggest next steps (assign to sales rep, create activity, send introduction email)

### Post-Creation Actions

After leads are in Odoo CRM:
- Assign to appropriate sales rep based on territory or industry
- Create follow-up activity (call or email) within 24 hours
- Add to relevant nurture sequence via email marketing
- Track lead progression through pipeline via analytics
