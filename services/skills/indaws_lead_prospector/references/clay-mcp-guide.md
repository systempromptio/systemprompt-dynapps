# Clay MCP Tools — Quick Reference

This guide documents all Clay MCP tools available for lead prospecting.
Use these tools through the ~~lead enrichment integration.

---

## Tool Reference

### find-and-enrich-company

**Purpose:** Look up a single company by domain or LinkedIn URL. Returns company data and a taskId for further enrichment.

**When to use:** Single company research, pre-meeting prep, account intelligence.

**Parameters:**
| Parameter | Required | Type | Description |
|-----------|----------|------|-------------|
| `companyIdentifier` | Yes | string | Company domain (e.g., "stripe.com") or LinkedIn company URL |
| `companyDataPoints` | No | array | Data points to enrich during search creation |

**Available companyDataPoints:**
- `{"type": "Headcount Growth"}`
- `{"type": "Recent News"}`
- `{"type": "Investors"}`
- `{"type": "Company Competitors"}`
- `{"type": "Company Customers"}`
- `{"type": "Tech Stack"}`
- `{"type": "Website Traffic"}`
- `{"type": "Open Jobs"}`
- `{"type": "Revenue Model"}`
- `{"type": "Annual Revenue"}`
- `{"type": "Latest Funding"}`
- `{"type": "Custom", "dataPointName": "Name", "dataPointDescription": "Description of what to research"}`

**Returns:** Company data + `taskId`

**Example:**
```
find-and-enrich-company(
  companyIdentifier: "empresa-target.es",
  companyDataPoints: [
    {"type": "Tech Stack"},
    {"type": "Annual Revenue"},
    {"type": "Headcount Growth"}
  ]
)
```

---

### find-and-enrich-contacts-at-company

**Purpose:** Search for contacts at a company by role, title, department, or location. Returns matching contacts and a taskId.

**When to use:** Finding decision makers at a target company, building contact lists by role.

**Parameters:**
| Parameter | Required | Type | Description |
|-----------|----------|------|-------------|
| `companyIdentifier` | Yes | string | Company domain or LinkedIn company URL |
| `contactFilters` | No | object | Filters to narrow contact search |
| `dataPoints` | No | object | Contact and/or company data points to enrich |

**Available contactFilters:**
| Filter | Type | Description |
|--------|------|-------------|
| `job_title_keywords` | string[] | Titles to include (e.g., ["CEO", "CFO"]) |
| `job_title_exclude_keywords` | string[] | Titles to exclude (e.g., ["Intern"]) |
| `profile_keywords` | string[] | Keywords anywhere in LinkedIn profile |
| `certification_keywords` | string[] | Certifications |
| `languages` | string[] | Profile languages |
| `school_names` | string[] | Schools attended |
| `current_role_min_months_since_start_date` | number | Min months in current role |
| `current_role_max_months_since_start_date` | number | Max months in current role (new hires) |
| `locations` | string[] | Locations to include (e.g., ["Spain", "Valencia"]) |
| `locations_exclude` | string[] | Locations to exclude |

**Available dataPoints:**
```json
{
  "contactDataPoints": [
    {"type": "Email"},
    {"type": "Summarize Work History"},
    {"type": "Find Thought Leadership"}
  ],
  "companyDataPoints": [
    {"type": "Tech Stack"},
    {"type": "Annual Revenue"}
  ]
}
```

**Returns:** Contact list + `taskId`

**Example:**
```
find-and-enrich-contacts-at-company(
  companyIdentifier: "empresa-target.es",
  contactFilters: {
    job_title_keywords: ["CEO", "Director General", "CFO", "CTO"],
    locations: ["Spain"]
  },
  dataPoints: {
    contactDataPoints: [{"type": "Email"}],
    companyDataPoints: [{"type": "Tech Stack"}]
  }
)
```

---

### find-and-enrich-list-of-contacts

**Purpose:** Find specific named people at their companies. Use when you know the person's name.

**When to use:** Looking up specific individuals, researching known contacts.

**Parameters:**
| Parameter | Required | Type | Description |
|-----------|----------|------|-------------|
| `contactIdentifiers` | Yes | array | Array of {contactName, companyIdentifier} objects |
| `dataPoints` | No | object | Contact and/or company data points to enrich |

**Returns:** Contact data + `taskId`

**Example:**
```
find-and-enrich-list-of-contacts(
  contactIdentifiers: [
    {"contactName": "Juan Garcia", "companyIdentifier": "empresa.es"},
    {"contactName": "Maria Lopez", "companyIdentifier": "otra-empresa.com"}
  ],
  dataPoints: {
    contactDataPoints: [{"type": "Email"}, {"type": "Summarize Work History"}]
  }
)
```

---

### add-company-data-points

**Purpose:** Enrich companies in an existing search with additional data points.

**When to use:** After initial search, when you need more company data.

**Parameters:**
| Parameter | Required | Type | Description |
|-----------|----------|------|-------------|
| `taskId` | Yes | string | Task ID from a previous search |
| `dataPoints` | Yes | array | Data points to add |
| `entityIds` | No | string[] | Specific company IDs to enrich (default: all) |

**Returns:** Confirmation. Use `get-existing-search` to poll for results.

---

### add-contact-data-points

**Purpose:** Enrich contacts in an existing search with additional data points.

**When to use:** After initial search, when you need emails, work history, or other contact data.

**Parameters:**
| Parameter | Required | Type | Description |
|-----------|----------|------|-------------|
| `taskId` | Yes | string | Task ID from a previous search |
| `dataPoints` | Yes | array | Data points to add |
| `entityIds` | No | string[] | Specific contact IDs to enrich (default: all) |

**Available contact data point types:**
- `{"type": "Email"}` — Find email addresses
- `{"type": "Summarize Work History"}` — Career summary
- `{"type": "Find Thought Leadership"}` — LinkedIn posts and articles
- `{"type": "Custom", "dataPointName": "Name", "dataPointDescription": "Description"}` — Custom research

**Returns:** Confirmation. Use `get-existing-search` to poll for results.

---

### get-existing-search

**Purpose:** Poll for enrichment results. Retrieves current state of a search including completed enrichments.

**When to use:** After adding data points, to check if enrichments are complete.

**Parameters:**
| Parameter | Required | Type | Description |
|-----------|----------|------|-------------|
| `taskId` | Yes | string | Task ID from a previous search |

**Polling pattern:**
1. Wait approximately 5 seconds after adding data points
2. Call `get-existing-search` with the taskId
3. Check each enrichment's `state` field:
   - `"in-progress"` — Still processing, wait and poll again
   - `"completed"` — Done, results in `value` field
4. If any enrichments are still in-progress, wait 5 seconds and poll again
5. Maximum 5 polling attempts (25 seconds total)

---

## Common Patterns for Indaws Prospecting

### Pattern 1: Industry-based prospecting

```
1. Identify target companies in industry (user provides domains or Clay finds them)
2. find-and-enrich-contacts-at-company for each company
3. add-company-data-points: Tech Stack + Annual Revenue
4. add-contact-data-points: Email + Work History
5. get-existing-search to poll results
6. Apply Quality Gate
7. Present ranked table
```

### Pattern 2: Single company deep-dive

```
1. find-and-enrich-company with domain + all relevant data points
2. find-and-enrich-contacts-at-company for decision makers
3. add-contact-data-points: Email + Work History + Thought Leadership
4. get-existing-search to poll results
5. Present enriched company profile
```

### Pattern 3: Named contact lookup

```
1. find-and-enrich-list-of-contacts with known names
2. add-contact-data-points: Email
3. get-existing-search to poll results
4. Apply Quality Gate
5. Present results
```

---

## Important Notes

- **Domain format:** Always use domain (e.g., "empresa.es"), not full URLs
- **Company names:** Convert known companies to domains before searching. If ambiguous, ask the user.
- **Rate limits:** Clay handles rate limiting internally. No need for manual delays.
- **Data freshness:** Clay data is generally current but verify critical information.
- **Custom data points:** Use for any research question not covered by standard types.
- **Entity IDs:** Use when enriching specific contacts/companies from a larger search to save credits.
