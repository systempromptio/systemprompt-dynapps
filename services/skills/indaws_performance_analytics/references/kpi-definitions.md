# KPI Definitions — Performance Analytics

## LinkedIn Organic

| Metric | Definition | Target cadence | Data source |
|--------|-----------|----------------|-------------|
| Impressions | Total times content was displayed | Weekly | LinkedIn Analytics (manual export) |
| Reach | Unique accounts that saw content | Weekly | LinkedIn Analytics |
| Engagement rate | (Reactions + Comments + Shares + Clicks) / Impressions | Weekly | LinkedIn Analytics |
| Click-through rate | Link clicks / Impressions | Per post | LinkedIn Analytics |
| Follower growth | Net new followers (gained - lost) | Weekly | LinkedIn Analytics |
| Post type performance | Engagement by format (text, image, carousel, document, video) | Monthly | LinkedIn Analytics |
| Best posting times | Engagement by day of week and hour | Monthly | LinkedIn Analytics |
| Profile views | Company page views | Weekly | LinkedIn Analytics |
| Share of voice | Indaws mentions vs competitor mentions | Monthly | Manual monitoring |

**Benchmarks — Odoo partner LinkedIn (Spanish market):**
- Engagement rate: 2–4% is good, >5% is excellent
- Follower growth: 2–5% monthly is healthy
- Best posting days: Tuesday, Wednesday, Thursday
- Best posting times: 08:30–09:30, 13:00–14:00

---

## Odoo Email Marketing

| Metric | Definition | Benchmark | Data source |
|--------|-----------|-----------|-------------|
| Sent | Total emails dispatched | — | Odoo Email Marketing |
| Delivered | Emails that reached inbox | >95% | Odoo Email Marketing |
| Open rate | Unique opens / Delivered | 20–30% (B2B Spain) | Odoo Email Marketing |
| Click rate | Unique clicks / Delivered | 2–5% (B2B Spain) | Odoo Email Marketing |
| Click-to-open rate | Unique clicks / Unique opens | 10–20% | Calculated |
| Bounce rate | Bounced / Sent | <2% hard, <5% total | Odoo Email Marketing |
| Unsubscribe rate | Unsubscribed / Delivered | <0.5% per campaign | Odoo Email Marketing |
| Reply rate | Replies / Delivered | Track only | Odoo Email Marketing |
| List growth rate | (New subscribers - Unsubscribes) / Total list | >2% monthly | Odoo Email Marketing |

Track by: campaign type (newsletter, product, event, nurture), automation flow, segment (industry, company size, lifecycle).

---

## SEO / Organic Search

| Metric | Definition | Target cadence | Data source |
|--------|-----------|----------------|-------------|
| Organic sessions | Visits from organic search | Weekly | Google Analytics |
| Keywords in top 10/20/50 | Count by ranking tier | Weekly | DataForSEO |
| Keyword positions | Position for target keywords | Weekly | DataForSEO |
| SERP CTR | Clicks / Impressions in Google Search | Monthly | Google Search Console |
| Indexed pages | Total pages indexed by Google | Monthly | Google Search Console |
| Core Web Vitals | LCP, FID/INP, CLS scores | Monthly | DataForSEO |
| Backlinks | Total referring domains, new/lost | Monthly | DataForSEO |
| Domain authority | Authority score | Monthly | DataForSEO |
| Organic traffic value | Estimated cost to acquire same traffic via ads | Monthly | DataForSEO |

---

## Events (Presencial + Webinars)

| Metric | Definition | Target | Data source |
|--------|-----------|--------|-------------|
| Invitations sent | Total invitations dispatched | — | Odoo Email Marketing |
| Registrations | Total sign-ups | >10% of invitations | Odoo Events / Forms |
| Attendance rate | Attended / Registered | >60% presencial, >40% webinar | Manual tracking |
| Leads generated | New contacts from event | Track per event | Odoo CRM |
| Pipeline value | Opportunities attributed to event | Track per event | Odoo CRM |
| Cost per lead | Total event cost / Leads generated | Track per event | Calculated |
| Event ROI | (Pipeline value - Event cost) / Event cost | >3x target | Calculated |
| Post-event engagement | Follow-up email opens/clicks, demo requests | Track per event | Odoo Email Marketing |
| NPS / Satisfaction | Attendee feedback score | >8/10 | Post-event survey |

---

## Website (www.indaws.es)

| Metric | Definition | Benchmark | Data source |
|--------|-----------|-----------|-------------|
| Sessions | Total website visits | Track trend | Google Analytics |
| Unique visitors | Distinct users | Track trend | Google Analytics |
| Pages per session | Average pages viewed per visit | >2.0 | Google Analytics |
| Bounce rate | Single-page sessions / Total sessions | <60% | Google Analytics |
| Avg session duration | Average time on site | >1:30 | Google Analytics |
| Conversion rate | Form submissions / Sessions | >2% | Google Analytics + Odoo |
| Demo requests | Demo/contact form submissions | Weekly target | Odoo CRM |
| Top landing pages | Highest-traffic entry pages | Monthly | Google Analytics |
| Top exit pages | Where visitors leave most | Monthly | Google Analytics |
| Mobile vs Desktop | Device distribution | Track split | Google Analytics |
| Geographic distribution | Sessions by Spanish region | Monthly | Google Analytics |

---

## Overall Pipeline (Marketing Attribution)

| Metric | Definition | Target cadence | Data source |
|--------|-----------|----------------|-------------|
| MQLs | Leads meeting qualification criteria from marketing | Weekly | Odoo CRM |
| SQLs | MQLs accepted by sales | Weekly | Odoo CRM |
| MQL-to-SQL conversion | SQLs / MQLs | Monthly, target >30% | Calculated |
| Opportunities created | New CRM opportunities from marketing leads | Monthly | Odoo CRM |
| Pipeline value | Total value of marketing-attributed opportunities | Monthly | Odoo CRM |
| Won deals (marketing) | Closed-won deals from marketing source | Monthly | Odoo CRM |
| Average deal size | Won revenue / Won deals | Quarterly | Odoo CRM |
| Sales cycle length | Days from MQL to Won | Quarterly | Odoo CRM |
| CAC | Total marketing spend / New customers acquired | Quarterly | Calculated |
| ROMI | (Revenue from marketing - Marketing cost) / Marketing cost | Quarterly | Calculated |
| LTV | Average customer revenue over relationship | Annually | Odoo CRM |
| LTV:CAC ratio | LTV / CAC | Annually, target >3:1 | Calculated |

---

## Data Sources Mapping

| Metric category | Primary data source | Access method | Update frequency |
|----------------|--------------------|--------------------|------------------|
| LinkedIn metrics | LinkedIn Analytics | Manual CSV export from company page | Weekly |
| Email campaign metrics | Odoo Email Marketing | odoo-pilot | Per campaign |
| SEO metrics | DataForSEO | seo-manager skill | Weekly |
| Website analytics | Google Analytics | Manual / GA4 | Weekly |
| Pipeline and CRM | Odoo CRM | odoo-pilot | Real-time |
| Event data | Odoo Events + CRM | odoo-pilot | Per event |
| Competitor SEO | DataForSEO | seo-manager skill | Monthly |
| Financial / Budget | Odoo Accounting | odoo-pilot | Monthly |

### Data Collection Checklist (Weekly)

- [ ] Export LinkedIn Analytics CSV (company page > Analytics > Export)
- [ ] Pull email campaign metrics from Odoo Email Marketing via odoo-pilot
- [ ] Run seo-manager keyword ranking check
- [ ] Review Google Analytics dashboard (sessions, conversions)
- [ ] Check Odoo CRM for new MQLs, SQLs, and pipeline changes
- [ ] Note any active campaigns or events for context
