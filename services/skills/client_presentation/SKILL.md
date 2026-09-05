# Client Presentation

You turn a brief, meeting notes, or a status report into a short slide deck the
user can open in any browser, step through with the arrow keys, and print to PDF.
The deck is one HTML file with no external assets, at most twelve slides, one
idea per slide, in Astound Digital branding. Every fact on a slide traces back to
the source material or a knowledge-bank hit.

## Use when

- "Turn this brief into a deck for the steering committee."
- "Make me five slides on where the project stands."
- "I need a short presentation for the Larkspur Outdoors QBR."
- Any time a client brief, prep pack, pipeline review, or stakeholder update
  needs to be presented rather than read.

## Inputs

- **Source material** (required): a file in the working folder (for example a
  `briefs/*.md` produced by Client Research or Meeting Prep), pasted notes, or a
  client name to research.
- **Audience and occasion** (required): who is in the room and what the meeting
  is for. This decides the tone and how much detail survives.
- **Slide count** (optional). Default is eight; never more than twelve.
- **Working folder** (optional). The deck is saved there; without one it is
  returned in chat as a code block with a note that it was not saved.

## Workflow

1. **Search the knowledge bank first.** Call the `knowledge-bank` MCP tool
   `search_knowledge` with `{query, top_k, source_types}` to confirm and enrich
   the source material, and note the `uri` and `date` of each hit you use:
   - `{"query": "<client> overview scope goals", "top_k": 6, "source_types": ["client_accounts"]}`
   - `{"query": "<client> milestones status risks", "top_k": 8, "source_types": ["project_docs", "project_tickets"]}`
   Use `client_accounts` for client facts and `project_docs` or
   `project_tickets` for delivery facts. Anything on a slide that is not in the
   source material or a hit does not go on the slide.
2. **Check whether Salesforce is linked.** If the `salesforce` server is
   available and the deck needs live commercial numbers (open opportunities,
   cases), use the Salesforce query tool read-only, for example
   `SELECT Name, StageName, Amount, CloseDate FROM Opportunity WHERE Account.Name = :name AND IsClosed = false`.
   If it is not linked, note "CRM half skipped: Salesforce is not linked" in the
   closing sources slide and continue with the knowledge bank.
3. **Outline before you build.** Write the slide list in chat: title, then one
   line per slide. The fixed shape is: title slide, agenda, one idea per body
   slide, closing next steps. If the outline exceeds twelve slides, merge or cut
   until it fits, then ask the user to confirm the outline if the material was
   ambiguous.
4. **Build the file** at `decks/<slug>.html`, where the slug is the client or
   topic in lower-case with hyphens plus the date, such as
   `decks/larkspur-outdoors-qbr-2026-09-05.html`. Requirements:
   - One file. No external CSS, JavaScript, fonts, or images. Logos or icons
     are inline SVG or omitted.
   - Each slide is a `<section class="slide">`; only one is visible at a time on
     screen. Navigation: right arrow, space, and page down go forward; left arrow
     and page up go back; Home and End jump to the first and last slide; a
     slide counter shows "n / total".
   - Print: a `@media print` block shows every slide, one per page
     (`page-break-after: always`), on a white background at A4 landscape.
   - Branding uses the repository colour tokens from `storage/files/css/core`:
     background `#f8f7f3` (brand-50), surfaces `#f1efed` (brand-100), text
     `#171717` (brand-700), headings `#0a0a0a` (brand-900), accent
     `oklch(0.615 0.245 29)` (the Astound red) for the title rule, the active
     agenda item, and one emphasis per slide at most. System font stack; no
     web fonts.
   - Title slide carries the deck title, the client name, "Astound Digital",
     the date, and "Confidential — prepared for <client>".
   - Body slides have a short heading, at most five bullets or one table or
     one number with context. Numbers carry scope and date on the slide.
   - Closing slide is "Next steps": owner, action, date, one line each.
   - Last slide is "Sources": the knowledge-bank hits and CRM queries used.
5. **Check it.** Open the file's HTML and confirm: the slide count matches the
   outline, there are no `http` references, the first slide is visible on load,
   and the printed order matches the on-screen order.
6. **Report** the path and the slide list, and ask the user to open the file in
   a browser to verify it before it goes anywhere.

## Result

- The path to `decks/<slug>.html` in the working folder.
- The slide list: number and title for each slide.
- A one-line note on anything left as a placeholder, in `{{...}}` form, that the
  user must fill before presenting.

Measure of done: the user opens the file, steps through it with the arrow keys,
prints it to PDF, and every slide reads correctly both ways.

## Guardrails

- **Read-only on the CRM.** Nothing in Salesforce is created or changed.
- **Never invent contacts, amounts, dates, or quotes.** A slide with no source
  becomes a `{{placeholder}}` and is listed in the report.
- **State the scope and date of every number on the slide itself**, not only in
  the sources slide. A chart or figure without a date is not presentable.
- **Twelve slides is the ceiling, one idea per slide is the rule.** If the
  material needs more, it is a document, not a deck; say so and offer to write it.
- **Brand name is "Astound Digital"** in full on every use; the URL form is
  astounddigital.com.
- **Confidential material** from `client_accounts` (margins, internal risk
  notes) stays off a client-facing deck unless the user confirms.
