# Salesforce Account Search

You read the CRM record of one named client through the signed-in user's
Salesforce hosted MCP — `salesforce-uk` for the UK org, `salesforce-us` for the
US org, or `salesforce-dev` when the session only has the test org — and return
it as dated evidence. This is a **building block**:
`client_360_report`, `account_status_report`, `account_suggestions` and
`pipeline_renewals_review` call the recipes below by name and never write
their own SOQL.

Everything runs as the caller's own Salesforce user. What they cannot see in
Salesforce, you cannot see here — report it as not visible, never as absent.

## Use when

- "Pull up Larkspur in Salesforce."
- "Who do we know at Larkspur and when did we last speak to them?"
- "What's open on the Larkspur account?"
- A report skill needs the `account`, `people`, `pipeline`, `service`,
  `activity` or `contracts` recipe.

## Inputs

- **Client** (required): name, website domain, or an 18-character Account Id.
- **Org** (optional): `salesforce-uk` by default; `salesforce-us` when the
  caller says so or the UK org has no match and the US one is connected.
  When only `salesforce-dev` is connected use it and say so in the evidence:
  it is a test org and its records are not a customer relationship.
- **Window** (optional): 90 days of activity and 12 months of closed
  opportunities by default.

## Step 0 — discover before you query

Salesforce's hosted MCP (`sobject-all`) exposes a small tool set whose exact
names can change between releases. As presented through the gateway on
2026-09-18 the ones you need are:

- `soqlQuery` — `{ "q": "<SOQL>" }`; the result carries `records`, `totalSize`
  and `done`, with `nextRecordsUrl` when a page remains,
- `getObjectSchema` — `{ "objects": "Account,Contact" }`, or no argument for
  the index; this is the describe,
- `find` — `{ "q": "FIND {Larkspur*} IN NAME FIELDS RETURNING Account(Id, Name, Website)" }`
  for free-text resolution when `LIKE` finds nothing,
- `getRelatedRecords`, `listRecentSobjectRecords`, `getUserInfo` — read-only
  helpers you may use; `createSobjectRecord`, `updateSobjectRecord`,
  `updateRelatedRecord`, `deleteSobjectRecord`, `deleteRelatedRecord` are
  the writes this skill never calls.

Confirm the names against the session's tool list on every run and use
whatever the server actually presents; never assume a name from memory. Then describe `Account`, `Contact`,
`Opportunity`, `Case`, `Task`, `Event` once and keep the field lists: the
queries below name standard fields, and a described custom field that answers
a report's question (`Health_Score__c`, `ARR__c`, `Renewal_Date__c`,
`Account_Tier__c`, `Champion__c`) is added to the relevant recipe. A field the
describe did not list is not queried. If describe is unavailable, fall back to
`SELECT FIELDS(STANDARD) FROM <object> ... LIMIT 200`.

If the SOQL tool rejects a subquery (`(SELECT ... FROM Contacts)`) run the
child query separately with `WHERE AccountId = :id`. Never send HTML entities
in SOQL; send `<`, `>`, `'` as they are.

## Step 1 — resolve the account

1. Id given → `SELECT Id, Name, Website FROM Account WHERE Id = '<id>'`.
2. Otherwise `SELECT Id, Name, Website, Type, Industry, OwnerId, Owner.Name,
   BillingCountry, LastActivityDate FROM Account WHERE Name LIKE '%<client>%'
   OR Website LIKE '%<domain>%' ORDER BY LastActivityDate DESC NULLS LAST
   LIMIT 20`.
3. One match → continue. Several → show them (name, website, owner, country,
   last activity) and **ask** which. Zero → say so; suggest the alias or
   domain; do not pick a look-alike.

Record `AccountId`, `Name`, `Website`, `OwnerName`. Then fix the currency
before any amount is read: if `Opportunity` describes a `CurrencyIsoCode`
field the org is multi-currency and every row carries its own code;
otherwise run `SELECT DefaultCurrencyIsoCode FROM Organization` and that one
code labels every amount in the run (a UK client in a USD org is reported in
USD). Every later query is scoped by the Account Id.

`LastActivityDate` on Account and Contact is the latest task or event date,
**scheduled ones included**, so it can sit in the future. It is never "last
contact": last contact is the newest *completed* task or *past* event from
the `activity` recipe.

## Recipes

Run only the recipes the caller asks for, in this order, each with the `LIMIT`
shown. Every amount carries its `CurrencyIsoCode`. When that column does not
exist the org has one currency: read it once with
`SELECT DefaultCurrencyIsoCode FROM Organization` and label every amount with
it — never assume GBP from the client's country. Never sum amounts across
currencies; total per currency and say so.

### `account`

`SELECT Id, Name, Type, Industry, Website, Phone, BillingCity, BillingCountry,
AnnualRevenue, NumberOfEmployees, Owner.Name, Owner.Email, ParentId,
Parent.Name, Description, CreatedDate, LastActivityDate, LastModifiedDate
FROM Account WHERE Id = '<id>'` — plus any described custom health, tier, ARR
or renewal field.

### `people`

`SELECT Id, Name, Title, Email, Phone, Department, ReportsToId, ReportsTo.Name,
LastActivityDate, CreatedDate FROM Contact WHERE AccountId = '<id>' ORDER BY
LastActivityDate DESC NULLS LAST LIMIT 100`.

Then, for the contact-role view: `SELECT ContactId, Contact.Name, Role,
IsPrimary, OpportunityId, Opportunity.Name FROM OpportunityContactRole WHERE
Opportunity.AccountId = '<id>' LIMIT 200`.

Classify each person from `Title` and `Role` only — Decision maker, Economic
buyer, Champion, Influencer, User, Unknown — and say the classification is
inferred from the title. A contact with no activity in 90 days is **stale**;
no activity ever is **never contacted**.

### `pipeline`

Open: `SELECT Id, Name, StageName, Amount, CurrencyIsoCode, CloseDate,
Probability, NextStep, Type, LeadSource, Owner.Name, CreatedDate,
LastModifiedDate, LastActivityDate FROM Opportunity WHERE AccountId = '<id>'
AND IsClosed = false ORDER BY CloseDate ASC LIMIT 100`.

Closed, last 12 months: `... WHERE AccountId = '<id>' AND IsClosed = true AND
CloseDate = LAST_N_MONTHS:12 ORDER BY CloseDate DESC LIMIT 100`, keeping
`IsWon`.

Stage velocity, when `OpportunityHistory` is readable: `SELECT OpportunityId,
StageName, Amount, CloseDate, CreatedDate FROM OpportunityHistory WHERE
Opportunity.AccountId = '<id>' AND Opportunity.IsClosed = false ORDER BY
CreatedDate DESC LIMIT 200`. Days in current stage = today minus the newest
history row for that stage; an open opportunity with no history row uses
`LastModifiedDate` and says so.

Products, when `OpportunityLineItem` is readable: `SELECT OpportunityId,
Product2.Name, Product2.Family, Quantity, TotalPrice, CurrencyIsoCode FROM
OpportunityLineItem WHERE Opportunity.AccountId = '<id>' LIMIT 200`.

An open opportunity whose `CloseDate` is in the past is **slipped**; one with
no `NextStep` and no activity in 30 days is **stalled**. Pipeline is not
revenue and is never turned into a forecast.

### `service`

`SELECT Id, CaseNumber, Subject, Status, Priority, Origin, Contact.Name,
Owner.Name, CreatedDate, ClosedDate, LastModifiedDate FROM Case WHERE
AccountId = '<id>' AND (IsClosed = false OR ClosedDate = LAST_N_DAYS:90)
ORDER BY IsClosed ASC, Priority ASC, CreatedDate DESC LIMIT 100`.

Open **High**/**Critical** cases older than seven days are a service risk;
name them in the evidence.

### `activity`

Past: `SELECT Id, Subject, Status, ActivityDate, Owner.Name, Who.Name,
What.Name, Description FROM Task WHERE AccountId = '<id>' AND ActivityDate =
LAST_N_DAYS:90 ORDER BY ActivityDate DESC LIMIT 100` and `SELECT Id, Subject,
ActivityDate, StartDateTime, Owner.Name, Who.Name, What.Name FROM Event WHERE
AccountId = '<id>' AND ActivityDate = LAST_N_DAYS:90 ORDER BY ActivityDate
DESC LIMIT 100`.

Scheduled: the same two queries with `ActivityDate = NEXT_N_DAYS:30 ORDER BY
ActivityDate ASC`.

Overdue: `SELECT Id, Subject, ActivityDate, Owner.Name, Who.Name FROM Task
WHERE AccountId = '<id>' AND IsClosed = false AND ActivityDate < TODAY ORDER BY
ActivityDate ASC LIMIT 50`.

From these derive, per account and per contact where a `WhoId` is present:
**last action** (newest completed task or past event: date, owner, subject)
and **next action** (soonest open task or future event: date, owner, subject,
and `overdue` when its date has passed). Absent either, say `none recorded`.

### `contracts`

Only when the objects are described and readable:
`SELECT Id, ContractNumber, Status, StartDate, EndDate, ContractTerm,
Owner.Name FROM Contract WHERE AccountId = '<id>' ORDER BY EndDate DESC LIMIT
50` and `SELECT Id, OrderNumber, Status, EffectiveDate, TotalAmount,
CurrencyIsoCode FROM Order WHERE AccountId = '<id>' ORDER BY EffectiveDate DESC
LIMIT 50`. A contract ending within 90 days is a **renewal**.

## Result: the evidence block

Return the resolved account line, then one block per recipe run, one row per
record, in this shape. Report skills copy it verbatim into their Sources.

```
account    | Larkspur Outdoors Ltd | 001xx000003ABCDEF | owner: A. Morgan | last activity 2026-09-12 | salesforce-uk
recipe     | record                               | key facts                                      | date       | id
pipeline   | FY27 Platform renewal                | Negotiation · GBP 240,000 · close 2026-11-30 · 34 days in stage · next step: legal review | 2026-09-10 | 006xx…
service    | Case 00045821 Checkout latency       | High · open 12 days · owner J. Patel           | 2026-09-06 | 500xx…
activity   | Call — renewal pricing (J. Hale)     | last action · owner A. Morgan                   | 2026-09-12 | 00Txx…
activity   | Task — send revised SOW (J. Hale)    | next action · due 2026-09-15 · OVERDUE          | 2026-09-15 | 00Txx…
```

Close with one line per recipe: `pipeline: 2 queries, 7 rows (LIMIT 100 not
reached)` or `contracts: not run — Contract not readable by this user`. Name
every cap you hit and every field you could not read.

## Rules

- **Read-only.** Never create, update or delete a record, never log an
  activity. If the user asks for a change, show it and wait for a clear yes.
- **Connection first.** If no Salesforce server is available or a call fails
  with an auth error, tell the user to connect the org on the Connections
  page and stop. Do not answer from memory or from a fixture.
- **Not found is not absent.** A `forbidden` or empty result means "not
  retrievable by this caller"; say that, never that the record does not exist.
- **Every number has a currency and a date.** Amounts carry the ISO code and
  the record's `LastModifiedDate`; totals are per currency.
- **Fixtures are synthetic.** A source path containing `fixtures`, a seed
  marker or a demo label cannot establish a real relationship; keep any worked
  example in a labelled simulation section.
- **Bound everything.** Every query carries a `LIMIT`; say when it was hit.
- Do not send CRM content to any external search or model outside this
  session.
