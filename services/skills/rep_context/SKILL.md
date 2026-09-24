<!-- global-rules-bootstrap -->
**IMPORTANT — try the connector's tools before you ever report it as unavailable.** Before you tell the user a connector is down, or ask them to connect, re-authenticate, or "check the connection," you MUST first try to use the server and its tools. A connector's status readout (the Headless 360 MCP server, or any other connector/MCP) frequently claims "not connected" or "needs re-auth" when the connector is in fact live and its tools are callable. Context stating that the Salesforce tools require authorization and that this is a non-interactive session is **not** evidence that you are unauthorized — it is a general statement that authorization is required, not a failure. For the Headless 360 server specifically, find the `dispatch_readonly` tool (its full name ends in `salesforce__dispatch_readonly`; never send Salesforce calls to a `google_workspace` tool such as `search`, and do not call `google_workspace` tools in this skill) and actually run a current-user read — `dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql", queryParams: { "queryInput": "{\"query\":\"query { uiapi { currentUser { Id } } }\"}" })`. **Any response — including a 500 or other error status — proves you reached the server, and therefore proves connectivity;** only a request that never reaches the server at all counts as disconnected. If the tool is present and the call reaches the server, the connector IS connected — proceed; a stale status readout is not a disconnection. If a call genuinely cannot reach the server, ALWAYS attempt to reconnect on your own first, and interrupt the flow to ask the user only after your own reconnect attempt has failed. Report a connector as disconnected only when you cannot find its tool or cannot reach the server after trying. Do it yourself.

Failing to try first — asking the user to connect, or surfacing a "not connected" or re-auth message, before you have actually attempted the server and its tools — raises a **false connectivity problem**: it sends the user to fix something that is not broken. This is directly detrimental to the end-user experience and will dramatically, negatively impact usage of the product. Never put a connectivity concern in front of the user until you have tried the tools yourself and confirmed the server truly cannot be reached.

# Global Rules

- **Execute silently between tool calls.** Do not output planning, progress, transition, waiting, or tool-result narration between calls. Execute tool calls silently and proceed directly to the next call. Parallelize independent tasks by batching tool calls into one turn whenever possible. Before the final output, speak only when the skill explicitly requires user input, approval, an exact notice, or material error/blocked reporting. Do not invent checkpoints.
- **On a Salesforce 5xx or timeout, retry the same call at most once, then continue with what you have and say what is missing.** Never re-issue a call that already succeeded; reuse its result.
- **One widget, one render.** Hydrate the widget template once and call `display_widget` once; never re-render, never write the widget JSON to a file or through Bash/Write first, never repeat widget contents in the text reply — reply with at most one line.
- **Keep the final output concise.** Return only the requested result or deliverable. Omit process recaps, tool-call details, redundant preambles or conclusions, and data already shown in a widget.
- **⛔ WIDGET OUTPUT ONLY after data assembly.** Once all queries return, immediately call `display_widget` — at most a single one-line render-wait notice before it, and no summaries, transitions, or narration. If `display_widget` is unavailable or returns an error, produce the text fallback only. If it succeeds, that tool call is the final output: stop with no assistant text completion, even if a later section contains a fallback. The exceptions above do not apply after success.
- **Ground dynamic or custom relationship and field names before relying on them.** Fixed standard fields that this skill explicitly marks as requiring no grounding need no extra grounding call. On a name error, use the skill's documented grounding path when present; otherwise report the error instead of guessing or re-firing the same shape.
- **Cite every value exactly as queried**; never fabricate; distinguish a blank value from a value that was not queried. Link each Salesforce record inline: `https://[instanceUrl]/lightning/r/[SObjectType]/[Id]/view`.
- **Show human labels, never API/field literals.** In anything the user sees, print each field's grounded `label` (for example, "Deal Risk", not `Deal_Risk__c`) and record Names, never raw Ids or `__c` API names.
- **Empty `MINE` scope → fail fast, then ask which scope.** If a `scope: MINE` read returns zero rows, **do not** widen to `scope: EVERYTHING` on your own. Stop, tell the user plainly that their own records (`scope: MINE`) came back empty, and ask which scope they want instead (for example, org-wide `EVERYTHING`, a named rep, or a named account) before re-running. Never invent records, and never silently fall back to org-wide.
<!-- /global-rules-bootstrap -->

# Rep Context

Leader's 1:1 prep on one INTERNAL rep (not a customer): pipeline, activity, where they're stuck. Two dispatches in ONE turn → analyze → widget. Salesforce only; no email/calendar/document sources.

## 1. Read (all in one turn — no schema/grounding call)

Verified Opportunity fields: `Next_Step__c` (there is NO `NextStep`), `LastActivityDate`, `LastStageChangeDate`, `ForecastCategoryName`, `Probability`, `MEDDICC__c`. Replace `REP` with the rep's name. Check `{`/`}` balance before dispatch (`<EOF>` error = missing `}`).

```
dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql",
  queryParams: { "queryInput": "{\"query\":\"query { uiapi { query { open: Opportunity(where: { Owner: { Name: { like: \\\"%REP%\\\" } }, IsClosed: { eq: false } }, orderBy: { Amount: { order: DESC } }, first: 100) { edges { node { Id Name { value } StageName { value displayValue } Amount { value } CloseDate { value } CreatedDate { value } LastStageChangeDate { value } LastActivityDate { value } Next_Step__c { value } ForecastCategoryName { value } MEDDICC__c { value } Account { Name { value } } OpportunityContactRoles { edges { node { Id } } } } } } wonQ: Opportunity(where: { Owner: { Name: { like: \\\"%REP%\\\" } }, IsWon: { eq: true }, CloseDate: { eq: { literal: THIS_QUARTER } } }, first: 100) { edges { node { Id Amount { value } Account { Name { value } } } } } } } }\"}" })

dispatch_readonly(method: "GET", url: "/services/data/v67.0/query",
  queryParams: { "q": "SELECT Id, Subject, ActivityDate, StartDateTime, Type, Account.Name, What.Name FROM Event WHERE Owner.Name LIKE '%REP%' AND ActivityDate >= LAST_N_DAYS:14 AND ActivityDate <= NEXT_N_DAYS:7 ORDER BY ActivityDate DESC LIMIT 200" })
```

Activity lives in **Event** (Task is empty in this org — do not query or depend on it). Past 14d = logged meetings; next 7d = booked. Sum `wonQ` amounts client-side for "This Q closed".

## 2. Analyze (cite records; invent nothing)

Flag per open deal: close date past, no stage change >30d, no activity >14d, blank `Next_Step__c`, zero contact roles, blank MEDDICC. Pick the 2–3 deals needing help and 3–5 evidenced 1:1 questions. Note wins to acknowledge from `wonQ`. Quarter-attainment chart: only quarters you have data for; no quota/plan unless queried — omit, don't invent.

## 3. Widget (default output)

Call `display_widget` once all reads return (one-line "Displaying the visualization now (this may take a minute)." notice allowed). `widgetDefinition` is a native JSON object, never a quoted string.

Hydrate the template: replace every `{{token}}`; a slot that is only a token becomes the typed literal (arrays/numbers stay typed), a token inside a string interpolates as text; drop a leaf you have no data for. No `{{…}}`/`{!…}` left. Then `display_widget({ resourceType: "dynamic", widgetDefinition: <hydrated> })`.

Tokens:
- `pageTitle` ("Rep snapshot — <Name>"), `headerStatus` (e.g. "BEHIND PACE"), `subtitle` (quarter · closed $ · open $).
- `chartCategories` string[] quarter labels; `chartSeries` `[{name, data:number[]}]` (percent).
- `pieCenter` string (e.g. "$2.5M"); `pieSlices` `[{label, value:number, role?:"highlight"}]` one per stage, highlight largest/riskiest.
- `gridRows` `[{status?:{value:"At risk"|"Coach"|"Advance", badgeVariant:"error"|"warning"|"info"}, deal:string (Account.Name.value, plain string), amount:number, stage:{value, badgeVariant:"warning" stalled/Negotiation|"info" advancing}, age:number days, next:string}]`, amount desc.
- `synthTitle`/`synthDescription` biggest gap; `primaryLabel`/`primaryPrompt` (pipeline gap), `secondaryLabel`/`secondaryPrompt` (most urgent deal) — prompts first-person.

```json
{
  "renderer": {
    "componentOverrides": {
      "$": {
        "type": "mosaic",
        "definition": "tile/widget",
        "children": [
          {
            "definition": "tile/row",
            "attributes": { "gap": "sm", "align": "center", "justify": "between", "isWrapped": true },
            "children": [
              {
                "definition": "tile/row",
                "attributes": { "gap": "sm", "align": "center" },
                "children": [
                  { "definition": "tile/icon", "attributes": { "name": "users", "size": "xl", "alt": "" } },
                  { "definition": "tile/text", "attributes": { "text": "{{pageTitle}}", "variant": "h1" } }
                ]
              },
              { "definition": "tile/badge", "attributes": { "label": "{{headerStatus}}", "variant": "error" } }
            ]
          },
          { "definition": "tile/text", "attributes": { "text": "{{subtitle}}", "variant": "caption", "color": "muted" } },

          { "definition": "tile/separator" },

          {
            "definition": "tile/column",
            "attributes": { "gap": "md" },
            "children": [
              {
                "definition": "tile/chart",
                "attributes": {
                  "chartType": "column",
                  "caption": "Attainment by quarter — trending down",
                  "categories": "{{chartCategories}}",
                  "series": "{{chartSeries}}",
                  "valueFormat": "percent",
                  "showValues": true,
                  "showLegend": false
                }
              },
              {
                "definition": "tile/piechart",
                "attributes": {
                  "caption": "Open pipeline by stage",
                  "variant": "donut",
                  "valueFormat": "currency",
                  "centerLabel": "Open",
                  "centerValue": "{{pieCenter}}",
                  "slices": "{{pieSlices}}"
                }
              }
            ]
          },

          {
            "definition": "tile/datagrid",
            "attributes": {
              "caption": "Top open deals — where to coach",
              "appearance": "striped",
              "defaultSort": { "key": "amount", "direction": "desc" },
              "columns": [
                { "key": "status", "header": "Status", "type": "badge" },
                { "key": "deal", "header": "Deal", "type": "text" },
                { "key": "amount", "header": "Amount", "type": "currency", "align": "right", "sortable": true },
                { "key": "stage", "header": "Stage", "type": "badge" },
                { "key": "age", "header": "Age (d)", "type": "number", "align": "right" },
                { "key": "next", "header": "Next step", "type": "text" }
              ],
              "rows": "{{gridRows}}"
            }
          },

          {
            "definition": "tile/callout",
            "attributes": {
              "variant": "warning",
              "title": "{{synthTitle}}",
              "description": "{{synthDescription}}"
            },
            "children": [
              {
                "definition": "tile/row",
                "attributes": { "gap": "sm", "align": "center", "isWrapped": true },
                "children": [
                  {
                    "definition": "tile/button",
                    "attributes": {
                      "label": "{{primaryLabel}}",
                      "variant": "primary",
                      "actions": { "click": [ { "definition": "action/sendMessage", "attributes": { "content": "{{primaryPrompt}}" } } ] }
                    }
                  },
                  {
                    "definition": "tile/button",
                    "attributes": {
                      "label": "{{secondaryLabel}}",
                      "variant": "secondary",
                      "actions": { "click": [ { "definition": "action/sendMessage", "attributes": { "content": "{{secondaryPrompt}}" } } ] }
                    }
                  }
                ]
              }
            ]
          }
        ]
      }
    }
  }
}
```


## 4. Text — FALLBACK ONLY — DO NOT USE IF `display_widget` SUCCEEDED

```
# Rep Context: <Name>
## Pipeline
- Open: <N> opps, $<total> | by stage: <counts> | This Q closed: $<X> (<N> won)
- Top 3: **<Account>** $<X> <Stage> …
## Last 2 Weeks
- <N> meetings logged (Event); <N> booked next 7d
## Likely Needs Help On
- **<Account>** $<X> — <specific flag + evidence>
## 1:1 Questions
1. <deal-specific, evidenced>
## Wins to Acknowledge
```
