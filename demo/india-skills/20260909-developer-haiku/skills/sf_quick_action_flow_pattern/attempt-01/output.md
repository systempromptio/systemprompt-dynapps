**Evidence Summary**

Designed Lead.Review Screen Flow action saved to `/home/tester/work/output/review.md`:

- **Screen Flow** (Lead_Review): recordId input variable, ReviewScreen as first screen with `allowBack=true`, UpdateLeadStatus action to set Status → Qualified
- **Quick Action** (Lead.Review): references Lead_Review flow by API name
- **Page Layout snippet**: platformActionList with qualified action `Lead.Review`
- **FlexiPage snippet**: actionInstance with visibility rule filtering on `{!Record.Status} EQUAL "Open - Not Contacted"`
- **Deployment sequence**: Flow → Quick Action → Layout → FlexiPage

All four components linked and documented. No unsupported operations; synthetic fixture context confirmed (no live Salesforce/GitHub connectors used).