# Quality Gate Scoring System

Migrated from the legacy outbound-lead-generator pipeline.
Applied to every lead generated via Clay MCP to assess data quality and relevance.

---

## Scoring Overview

Every lead starts with a **base score of 50** (neutral).
Checks are applied sequentially, adding or subtracting points.
Final score determines classification: PASS, WARN, or FAIL.

---

## Scoring Table

| # | Check | Condition | Impact | Score change |
|---|-------|-----------|--------|-------------|
| 1 | **Blacklisted domain** | Domain matches blacklist (see below) | Auto-FAIL | -100 |
| 2 | **Non-Spanish country** | Company location country is not Spain (ES) | Penalty | -30 |
| 3 | **Spanish country** | Company location confirmed as Spain (ES) | Bonus | +15 |
| 4 | **Spanish TLD** | Domain ends with .es | Bonus | +15 |
| 5 | **.com TLD** | Domain ends with .com | Small bonus | +5 |
| 6 | **Suspicious TLD** | Domain ends with .info, .xyz, .top, .ru, .cn, .jp, .su, .tk, .ml, .ga, .cf | Penalty | -25 |
| 7 | **Company name matches domain** | Significant words from company name found in domain | Bonus | +15 |
| 8 | **Email domain mismatch** | Contact email domain does not match company website domain | Penalty | -5 |
| 9 | **Has verified email** | Clay returned a verified email address for contact | Bonus | +10 |
| 10 | **Has LinkedIn profile** | Contact has a LinkedIn profile URL | Bonus | +5 |
| 11 | **ICP industry match** | Company industry matches Indaws ICP industries | Bonus | +10 |

---

## Thresholds

| Classification | Score range | Action |
|---------------|------------|--------|
| **PASS** | >= 60 | Ready for outreach. Import to Odoo CRM. |
| **WARN** | >= 35 and < 60 | Manual review recommended. May import with lower priority. |
| **FAIL** | < 35 | Do not import. Data quality or relevance too low. |

---

## Scoring Examples

**Example 1: High-quality Spanish lead**
```
Base score:                           50
+ Spanish country (ES):             +15 → 65
+ Spanish TLD (.es):                +15 → 80
+ Company name matches domain:      +15 → 95
+ Has verified email:               +10 → 105 (capped display at 100)
+ ICP industry match:               +10 → 115
= Final: 100 (capped) → PASS
```

**Example 2: Moderate lead with some issues**
```
Base score:                           50
+ Spanish country (ES):             +15 → 65
+ .com TLD:                          +5 → 70
- Email domain mismatch:             -5 → 65
+ Has LinkedIn profile:              +5 → 70
= Final: 70 → PASS
```

**Example 3: Low-quality international lead**
```
Base score:                           50
- Non-Spanish country:              -30 → 20
+ .com TLD:                          +5 → 25
- Email domain mismatch:             -5 → 20
= Final: 20 → FAIL
```

**Example 4: Blacklisted domain**
```
Base score:                           50
- Blacklisted domain:              -100 → -50
= Final: -50 → FAIL (auto)
```

---

## Domain Blacklist

### Categories

**Government domains:**
- `.gob.es` — Spanish government
- `.gov` — Any government
- `.gov.es` — Spanish government alternative
- `junta-*.es` — Regional government bodies (Juntas)
- `diputacion*.es` — Provincial government
- `ayuntamiento*.es`, `ajuntament*.es` — Municipal government

**Education domains:**
- `.edu` — Educational institutions
- `.edu.es` — Spanish educational institutions
- `.ac.uk`, `.ac.*` — Academic institutions
- `universidad*.es`, `universitat*.es` — Universities

**Media and news portals:**
- `elpais.com`, `elmundo.es`, `abc.es`, `lavanguardia.com`
- `expansion.com`, `cincodias.elpais.com`
- `europapress.es`, `efe.com`
- Any major news outlet domain

**Business directories and aggregators:**
- `einforma.com` — Spanish business directory
- `axesor.es` — Business information provider
- `infocif.es` — Company information portal
- `empresia.es` — Company directory
- `paginasamarillas.es` — Yellow pages
- `qdq.com` — Business directory
- `europages.es` — B2B directory
- `kompass.com` — Business directory

**Social media platforms:**
- `linkedin.com`, `facebook.com`, `twitter.com`, `instagram.com`
- `youtube.com`, `tiktok.com`, `pinterest.com`

**Tech giants and generic domains:**
- `google.com`, `google.es`, `microsoft.com`, `apple.com`
- `amazon.com`, `amazon.es`
- `wikipedia.org`

**Email providers (when used as company domain):**
- `gmail.com`, `hotmail.com`, `outlook.com`, `yahoo.com`
- `live.com`, `msn.com`, `icloud.com`

### Blacklist Maintenance

- Review and update blacklist quarterly
- Add new domains when encountered during prospecting
- Remove domains if a legitimate company coincidentally matches (rare)
- The blacklist is intentionally broad; false positives are caught at the FAIL threshold

---

## Quality Gate Application Process

### When Using Clay MCP Results

1. Receive lead data from `get-existing-search` results
2. For each lead, initialize score at 50
3. Apply each check in order (1-11)
4. If check #1 (blacklist) triggers, immediately classify as FAIL
5. Sum all score changes
6. Classify based on thresholds
7. Record all applied checks and their impacts for transparency
8. Present results sorted by score (highest first)

### Score Transparency

Always show the user how each lead was scored. Include in output:

```markdown
**Score breakdown for [Company Name]:**
- Base: 50
- [Check name]: [+/- X] (reason)
- [Check name]: [+/- X] (reason)
- **Total: [Score] → [PASS/WARN/FAIL]**
```

This transparency allows the user to override scores when they have additional context
(e.g., a personal referral to a company that scores WARN due to missing data).

---

## Manual Override Guidelines

The Quality Gate is a data quality filter, not a business judgment. Users may override:

| Override scenario | When appropriate |
|------------------|-----------------|
| WARN → PASS | User has personal connection, referral, or additional context |
| FAIL → WARN | Domain flagged incorrectly, user confirms legitimacy |
| PASS → FAIL | User knows company is not a fit (already a client, competitor, etc.) |

All overrides should be documented with the reason when creating the Odoo CRM lead.
