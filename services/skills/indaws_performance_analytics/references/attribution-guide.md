# Attribution Guide — UTM Tracking & Odoo Analytics

## UTM Parameters (Odoo native support)

Every marketing link must include:

| Parameter | Purpose | Example values |
|-----------|---------|----------------|
| `utm_source` | Where traffic comes from | linkedin, email, google, event, blog, partner |
| `utm_medium` | Marketing medium | organic, email, cpc, social, referral, event |
| `utm_campaign` | Specific campaign name | q1-2026-erp-webinar, newsletter-march-2026 |

---

## UTM Naming Convention for Indaws

**utm_source values:**
- `linkedin` — LinkedIn organic posts
- `email` — Odoo email marketing campaigns
- `google` — Google organic or paid
- `event-[name]` — Specific event (e.g., `event-odoo-experience-2026`)
- `blog` — Blog post CTAs
- `partner` — Dynapps or partner referrals
- `direct` — Direct/typed traffic

**utm_medium values:**
- `organic` — Organic/unpaid content
- `email` — Email campaigns
- `cpc` — Paid search (if used)
- `social` — Social media (generic)
- `referral` — Referral links
- `event` — Event-related traffic

**utm_campaign naming — format:** `[quarter]-[year]-[descriptive-name]`

Examples:
- `q1-2026-manufacturing-nurture`
- `q2-2026-erp-migration-guide`
- `q3-2026-september-event-valencia`

---

## How Attribution Works in Odoo

1. Visitor clicks a link with UTM parameters
2. Odoo captures UTM data in the website session
3. When visitor submits a form, UTM data is stored on the lead/opportunity
4. CRM reports can filter by source, medium, and campaign
5. Revenue from won deals can be attributed back to marketing source

**Attribution model:** First-touch (Odoo default). The first UTM source that created the lead gets full credit.

---

## Attribution Limitations

| Limitation | Impact | Workaround |
|-----------|--------|------------|
| First-touch only | Ignores nurture touchpoints | Track email engagement on existing leads separately |
| No cross-device tracking | Mobile visitor who converts on desktop appears as new | Accept as limitation, focus on trends |
| Phone/offline leads | No automatic attribution | Manually tag source when creating leads in CRM |
| LinkedIn DMs | Not trackable via UTM | Tag manually with source "linkedin" |
| Event leads | Scanned badges not linked | Import event attendees with campaign tag |
| Organic search | No keyword-level attribution | Use Google Search Console for keyword insights |

---

## Multi-Touch Considerations

While Odoo supports first-touch only, track these touchpoints manually:
- Email engagement history (opens, clicks per lead)
- Website page views per lead (Odoo website tracking)
- Event attendance per lead
- Content downloads per lead

For strategic decisions, supplement first-touch data with qualitative input:
- Ask "How did you hear about us?" in forms
- Sales team notes on what influenced the deal
- Post-close customer survey

---

## Odoo Analytics Fields

Relevant CRM fields for attribution reporting via odoo-pilot:
- `utm_source_id` — UTM source (Many2one to `utm.source`)
- `utm_medium_id` — UTM medium (Many2one to `utm.medium`)
- `utm_campaign_id` — UTM campaign (Many2one to `utm.campaign`)
- `source_id` — Lead source (alternative field on `crm.lead`)
- `expected_revenue` — Opportunity value
- `probability` — Win probability (%)
- `stage_id` — Current pipeline stage

For Odoo connection setup see: `references/shared/odoo-connection.md`
