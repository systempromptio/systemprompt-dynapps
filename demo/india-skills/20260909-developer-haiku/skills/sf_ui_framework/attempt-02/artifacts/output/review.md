# UI Framework Exercise: Lead Display LWC

## Artifacts Created

### Component Files
- **leadDisplay.html** — SLDS markup with semantic labels, status badge, loading spinner, error alert
- **leadDisplay.js** — LDS `getRecord` wire adapter; loads Lead Name + Status fields; state management for loading/error
- **leadDisplay.css** — Component-scoped SLDS token usage; badge styling; spinner container
- **leadDisplay.js-meta.xml** — Exposed to lightning__RecordPage/AppPage; recordId property config

### Test Suite
- **leadDisplay.test.js** — Jest + @salesforce/sfdx-lwc-jest; 11 test cases covering:
  - **Load state:** Spinner display, data render
  - **Error state:** Alert display, error recovery
  - **Keyboard:** Button `aria-label`, Enter key activation, focus retention, Tab order
  - **Accessibility:** Semantic HTML, ARIA role=alert/status, SLDS classes

## Accessibility Patterns Applied

✓ **Every interactive element labeled** — button has `aria-label="Refresh lead data"`  
✓ **Keyboard navigation** — refresh button responds to Enter/Space via native HTML  
✓ **WCAG AA contrast** — SLDS badge and form tokens ensure compliant colors  
✓ **Role attributes** — `role="alert"` on error, `role="status"` on loading state  
✓ **No text-only state** — badge uses SLDS color class + visible text  

## LDS Implementation
- Imported `getRecord` + `getFieldValue` from `lightning/uiRecordApi`  
- Wire adapter `@wire(getRecord, { recordId: '$recordId', fields: FIELDS })`  
- Reactive field list `['Lead.Name', 'Lead.Status']`  
- No custom Apex CRUD — pure LDS

## Saved Paths
```
/home/tester/work/output/leadDisplay.html
/home/tester/work/output/leadDisplay.js
/home/tester/work/output/leadDisplay.css
/home/tester/work/output/leadDisplay.js-meta.xml
/home/tester/work/output/leadDisplay.test.js
```

**No unsupported operations encountered.** All files created successfully.
