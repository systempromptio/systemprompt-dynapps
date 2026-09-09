**Saved to:** `/home/tester/work/output/review.md`

## Summary

Designed unsent Lead routing notification for SalesQueue_Distribution queue assignment. Includes:

1. **Letterhead, Email Folder, Template** (with body and metadata)
2. **Record-Triggered Flow** (64.0 API, auto-launched, creates/updates detection)
3. **Recipient Selection:** `Lead.Email` guarded by non-null check
4. **Transaction Behavior:** Insert/update to target queue sends; other owner changes/blank email skip silently
5. **Six Test Cases** (MT-01 through MT-06): positive send, wrong queue, email change, blank email, invalid format, delivery failure — all marked **NOT RUN**
6. **Fault Paths:** Guard vs. failure distinction (decision blocks send; platform handles delivery errors)

All artifacts defined as XML; no Salesforce org accessed. Email not sent or validated. Configuration complete and ready for review.