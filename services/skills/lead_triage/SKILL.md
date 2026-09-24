<!-- global-rules-bootstrap -->
**IMPORTANT — try the connector's tools before you ever report it as unavailable.** Before you tell the user a connector is down, or ask them to connect, re-authenticate, or "check the connection," you MUST first try to use the server and its tools. A connector's status readout (the Headless 360 MCP server, or any other connector/MCP) frequently claims "not connected" or "needs re-auth" when the connector is in fact live and its tools are callable. Context stating that the Salesforce tools require authorization and that this is a non-interactive session is **not** evidence that you are unauthorized — it is a general statement that authorization is required, not a failure. For the Headless 360 server specifically, find the `dispatch_readonly` tool and actually run a current-user read — `dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql", queryParams: { "queryInput": "{\"query\":\"query { uiapi { currentUser { Id } } }\"}" })`. **Any response — including a 500 or other error status — proves you reached the server, and therefore proves connectivity;** only a request that never reaches the server at all counts as disconnected. If the tool is present and the call reaches the server, the connector IS connected — proceed; a stale status readout is not a disconnection. If a call genuinely cannot reach the server, ALWAYS attempt to reconnect on your own first, and interrupt the flow to ask the user only after your own reconnect attempt has failed. Report a connector as disconnected only when you cannot find its tool or cannot reach the server after trying. Do it yourself.

Failing to try first — asking the user to connect, or surfacing a "not connected" or re-auth message, before you have actually attempted the server and its tools — raises a **false connectivity problem**: it sends the user to fix something that is not broken. This is directly detrimental to the end-user experience and will dramatically, negatively impact usage of the product. Never put a connectivity concern in front of the user until you have tried the tools yourself and confirmed the server truly cannot be reached.

# Global Rules

- **Execute silently between tool calls.** Do not output planning, progress, transition, waiting, or tool-result narration between calls. Execute tool calls silently and proceed directly to the next call. Parallelize independent tasks by batching tool calls into one turn whenever possible. Before the final output, speak only when the skill explicitly requires user input, approval, an exact notice, or material error/blocked reporting. Do not invent checkpoints.
- **Keep the final output concise.** Return only the requested result or deliverable. Omit process recaps, tool-call details, redundant preambles or conclusions, and data already shown in a widget.
- **⛔ WIDGET OUTPUT ONLY after data assembly.** Once all queries return, immediately call `display_widget` — at most a single one-line render-wait notice before it, and no summaries, transitions, or narration. If `display_widget` is unavailable or returns an error, produce the text fallback only. If it succeeds, that tool call is the final output: stop with no assistant text completion, even if a later section contains a fallback. The exceptions above do not apply after success.
- **Ground dynamic or custom relationship and field names before relying on them.** Fixed standard fields that this skill explicitly marks as requiring no grounding need no extra grounding call. On a name error, use the skill's documented grounding path when present; otherwise report the error instead of guessing or re-firing the same shape.
- **Cite every value exactly as queried**; never fabricate; distinguish a blank value from a value that was not queried. Link each Salesforce record inline: `https://[instanceUrl]/lightning/r/[SObjectType]/[Id]/view`.
- **Show human labels, never API/field literals.** In anything the user sees, print each field's grounded `label` (for example, "Deal Risk", not `Deal_Risk__c`) and record Names, never raw Ids or `__c` API names.
- **Empty `MINE` scope → fail fast, then ask which scope.** If a `scope: MINE` read returns zero rows, **do not** widen to `scope: EVERYTHING` on your own. Stop, tell the user plainly that their own records (`scope: MINE`) came back empty, and ask which scope they want instead (for example, org-wide `EVERYTHING`, a named rep, or a named account) before re-running. Never invent records, and never silently fall back to org-wide.
<!-- /global-rules-bootstrap -->

# Lead Triage

Resolve lead(s) → read Lead + matching Account + prior activity in one turn → score fit and intent → route. Salesforce only: no web search, email or Drive lookups (not available). All fields below are verified on this org — no `objectInfos` grounding calls.

## 1. Read (one turn, parallel)

SOQL via `dispatch_readonly(method: "GET", url: "/services/data/v67.0/query", queryParams: { "q": "<SOQL>" })`.

**Lead** — `<LEAD_FILTER>` from user input: `Id = '<id>'`, `Email = '<email>'`, or `Name LIKE '%<name>%' AND Company LIKE '%<company>%'`. For a queue ("my new leads") use `OwnerId = '<currentUserId>' AND IsConverted = false AND CreatedDate = LAST_N_DAYS:30`, `LIMIT 50`.
```
SELECT Id, Name, Company, Title, Email, Website, Industry, NumberOfEmployees, AnnualRevenue, LeadSource, Status, Rating, CreatedDate, LastActivityDate, IsConverted, QualificationScore__c, Lead_Score__c, Deal_Size__c, Market_segment__c, Business_Challenge__c, Requested_demo__c, Attended_event_or_webinar__c, Responded_to_Outreach__c, Next_Step__c, Owner.Name FROM Lead WHERE <LEAD_FILTER> ORDER BY CreatedDate DESC LIMIT 5
```
(Lead `Owner` is polymorphic — use SOQL `Owner.Name`; GraphQL `Owner { Name }` fails on Lead.)

**Account** (existing relationship check; Account has NO `Type` — use `Account_Type__c`):
```
SELECT Id, Name, Account_Type__c, Industry, Market_segment__c, Employee_Range__c, Client_Tier__c, Account_Health__c, Owner.Name FROM Account WHERE Name LIKE '%<COMPANY>%' OR Website LIKE '%<DOMAIN>%' LIMIT 5
```

**Prior engagement** (once the lead Id is known; activity lives in Event — Task is usually empty, query both):
```
SELECT Subject, ActivityDateTime, Owner.Name FROM Event WHERE WhoId IN (<leadIds>) ORDER BY ActivityDateTime DESC LIMIT 10
SELECT Subject, ActivityDate, Status FROM Task WHERE WhoId IN (<leadIds>) ORDER BY ActivityDate DESC LIMIT 10
```

Account with an owner → existing relationship: routing = coordinate with that owner.

## 2. Score

**ICP fit** (ask once if the ICP is not inferable from org data): Industry, Size (NumberOfEmployees / Employee_Range__c / AnnualRevenue), Persona (Title), Disqualifiers (e.g. Reason_Disqualified__c set, IsConverted). Each ✅/⚠️/❌ with the Salesforce value as evidence.

**Intent:** Source quality (High: demo request/referral · Med: content/event · Low: list/cold) · Specificity (Business_Challenge__c, Requested_demo__c) · Prior engagement (Event/Task, Responded_to_Outreach__c) · Scores (Lead_Score__c, QualificationScore__c, Rating).

**Priority** (P0 ≈ top 20%, P1 next 25%, P2 rest): P0 strong fit AND high intent · P1 strong fit + medium intent, or moderate fit + high intent · P2 moderate fit + low intent, or unclear · DQ hard disqualifier. Widget tiers: P0 = Hot, P1 = Warm, P2 = Nurture.

## 3. Render

Call `display_widget({ resourceType: "dynamic", widgetDefinition: <hydrated template> })` immediately — `widgetDefinition` is a native JSON object, never a quoted string. Replace every `{{token}}` with a literal (no `{{…}}`/`{!…}` left). A slot that is only a token becomes the typed value (numbers stay numbers, arrays stay arrays); a token inside a string interpolates as text; omit keys you have no data for.

Tokens:
- `headerTitle` ("Lead triage — 23 new leads"), `headerSubtitle` ("Scored by fit and intent · 5 hot, 9 warm, 9 nurture").
- `centerValue` (string lead total); `slices`: `[{label:"Hot",value:5,role:"highlight"},{label:"Warm",value:9},{label:"Nurture",value:9}]`.
- `donutAsideTitle`; `donutAside`: one sentence (18–32 words) naming every slice with count and % (legend is off), insight first.
- `chartAsideTitle`; `chartAside`: one sentence on which sources dominate. `categories`: LeadSource names; `series`: `[{name:"Leads",data:[numbers matching categories]}]`.
- `totalRows` (number); `hotRows`: `[{status:{value:"Hot",badgeVariant:"success"}, lead, co, score (0–100), src:{value,badgeVariant}, next}]` — src badge `success` for demo/referral, `info` or `secondary` for webinar/event/content.
- `calloutTitle`/`calloutDescription`: the queue's single most important read (don't repeat aside claims). `ctaPrimaryLabel`/`ctaPrimaryMsg`: first-person prompt, e.g. "Draft outreach to Beacon Health (J. Okafor)…". `salesforceUrl`: `https://<myDomain>/lightning/o/Lead/list`.
- Quote blank fields as blank; drop leads with no data; never fabricate.

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
            "attributes": {
              "gap": "sm",
              "align": "center",
              "isWrapped": false
            },
            "children": [
              {
                "definition": "tile/icon",
                "attributes": {
                  "name": "user",
                  "size": "xl",
                  "alt": ""
                }
              },
              {
                "definition": "tile/text",
                "attributes": {
                  "text": "{{headerTitle}}",
                  "variant": "h1"
                }
              }
            ]
          },
          {
            "definition": "tile/text",
            "attributes": {
              "text": "{{headerSubtitle}}",
              "variant": "caption",
              "color": "muted"
            }
          },
          {
            "definition": "tile/separator"
          },
          {
            "definition": "tile/row",
            "attributes": {
              "gap": "lg",
              "align": "center",
              "isWrapped": false
            },
            "children": [
              {
                "definition": "tile/column",
                "attributes": {
                  "width": "auto"
                },
                "children": [
                  {
                    "definition": "tile/piechart",
                    "attributes": {
                      "caption": "Leads by tier",
                      "variant": "donut",
                      "valueFormat": "number",
                      "showLegend": false,
                      "centerLabel": "New leads",
                      "centerValue": "{{centerValue}}",
                      "slices": "{{slices}}"
                    }
                  }
                ]
              },
              {
                "definition": "tile/column",
                "attributes": {
                  "gap": "sm",
                  "align": "start",
                  "width": "stretch"
                },
                "children": [
                  {
                    "definition": "tile/text",
                    "attributes": {
                      "text": "{{donutAsideTitle}}",
                      "variant": "section-title"
                    }
                  },
                  {
                    "definition": "tile/text",
                    "attributes": {
                      "text": "{{donutAside}}",
                      "variant": "body",
                      "color": "muted"
                    }
                  }
                ]
              }
            ]
          },
          {
            "definition": "tile/text",
            "attributes": {
              "text": "{{chartAsideTitle}}",
              "variant": "section-title"
            }
          },
          {
            "definition": "tile/text",
            "attributes": {
              "text": "{{chartAside}}",
              "variant": "body",
              "color": "muted"
            }
          },
          {
            "definition": "tile/chart",
            "attributes": {
              "chartType": "column",
              "caption": "Leads by source",
              "categories": "{{categories}}",
              "series": "{{series}}",
              "valueFormat": "number",
              "showValues": true,
              "showLegend": false
            }
          },
          {
            "definition": "tile/datagrid",
            "attributes": {
              "caption": "Work these first — hot leads by score",
              "appearance": "striped",
              "defaultSort": {
                "key": "score",
                "direction": "desc"
              },
              "totalRows": "{{totalRows}}",
              "columns": [
                {
                  "key": "status",
                  "header": "Status",
                  "type": "badge"
                },
                {
                  "key": "lead",
                  "header": "Lead",
                  "type": "text"
                },
                {
                  "key": "co",
                  "header": "Company",
                  "type": "text"
                },
                {
                  "key": "score",
                  "header": "Fit score",
                  "type": "databar",
                  "target": 100,
                  "sortable": true
                },
                {
                  "key": "src",
                  "header": "Source",
                  "type": "badge"
                },
                {
                  "key": "next",
                  "header": "Next step",
                  "type": "text"
                }
              ],
              "rows": "{{hotRows}}"
            }
          },
          {
            "definition": "tile/callout",
            "attributes": {
              "variant": "info",
              "title": "{{calloutTitle}}",
              "description": "{{calloutDescription}}"
            },
            "children": [
              {
                "definition": "tile/row",
                "attributes": {
                  "gap": "sm",
                  "align": "center",
                  "isWrapped": true
                },
                "children": [
                  {
                    "definition": "tile/button",
                    "attributes": {
                      "label": "{{ctaPrimaryLabel}}",
                      "variant": "primary",
                      "actions": {
                        "click": [
                          {
                            "definition": "action/sendMessage",
                            "attributes": {
                              "content": "{{ctaPrimaryMsg}}"
                            }
                          }
                        ]
                      }
                    }
                  },
                  {
                    "definition": "tile/button",
                    "attributes": {
                      "label": "View in Salesforce",
                      "iconName": "open-in-new",
                      "variant": "secondary",
                      "actions": {
                        "click": [
                          {
                            "definition": "action/openLink",
                            "attributes": {
                              "url": "{{salesforceUrl}}"
                            }
                          }
                        ]
                      }
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

## 4. Text fallback (only if display_widget is unavailable or errors)

```
# Lead Triage: [Name] @ [Company]
**Priority: P[0/1/2/DQ]** — [one-sentence rationale]
## CRM Status: [Net new | Account exists — owned by [Name], coordinate first]
## ICP Fit: [Strong/Moderate/Poor]  [fit table]
## Intent: [High/Medium/Low]  [intent table]
## Recommended Action
- Route to: [self / owner / DQ queue] · SLA: [P0 same day / P1 48h / P2 this week]
- First touch: [hook or qualification questions or polite DQ]
## Suggested SFDC updates: Lead Status → … · Lead Owner → … · Next Step → "[triage summary]"
```
