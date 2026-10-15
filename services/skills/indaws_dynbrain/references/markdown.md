# The Markdown a document can actually be written in

A documentation is Markdown, but **not every Markdown**. What a page may contain is a
**closed grammar**: the set of constructs Odoo's editor can render AND turn back into the
same file. Anything outside it is not "unsupported and ignored" — it degrades, or it blocks
the save.

**Why you have to care even when you only write through the MCP.** Your `docs_create` /
`docs_update` writes the Markdown straight through: nothing on this page happens at that
moment. It happens **the first time a person opens that page in Odoo's editor**, because
the editor renders the file to HTML and serialises it back. From then on, the file in the
repository is whatever that round trip produced. So you are not writing for the MCP. You are
writing for the round trip, and it comes for every page eventually.

Measured on 310 real files of two repositories: 138 come back byte for byte, 170 come back
with the rewrites of the last section, and 2 cannot be saved at all.

## What survives, byte for byte

Write these freely. Each one has been measured going out to HTML and back unchanged.

| Construct | Syntax |
| --- | --- |
| Headings, paragraphs | `# H1` … `###### H6` |
| Bold, italic, both, strikethrough | `**b**`, `*i*`, `***bi***`, `~~gone~~` |
| Code span, and one containing a backtick | `` `code` ``, ``` `` `x` `` ``` |
| Nested lists, bullet and ordered | four spaces per level |
| Fenced code, with or without a language | ` ```bash ` |
| Diagram | a ` ```mermaid ` fence |
| Table, with per-column alignment | `\| :--- \| :---: \| ---: \|` |
| Admonition | `!!! warning "Title"` + 4-space body |
| **Copy-to-clipboard block** | `!!! template "Subject"` + 4-space body |
| Content tabs | `=== "Label"` + 4-space body |
| Collapsible, closed or open | `??? "Label"` / `???+ "Label"` |
| Video | `@[youtube](ID)`, `@[vimeo](ID)` |
| Blockquote, nested, and a rule | `>`, `> >`, `---` |
| Definition list | `Term` then `:   definition` |
| Raw HTML block holding Markdown | `<div markdown="1">…</div>` |
| Link to another page | `[text](other-page.md)`, `[text](other-page.md#anchor)` |
| Image or file next to the page | `![alt](assets/diagram.png)` |
| External link, with an optional title | `[text](https://… "Title")` |
| A literal `#`, `*`, `_`, `|` in prose | escape it: `\#`, `\*`, `\_`, `\|` |

Three notes worth carrying:

- **An admonition title is inline Markdown too**: `!!! tip "Use the **official** [guide](g.md)"`
  keeps its code spans, emphasis and links. A `"` inside the title is fine. (Before
  `dynapps_markdown` 19.0.1.2.0 this title was flattened and the words were glued together —
  if you meet a `"Usasiemprelaguía"` in an old file, that is what happened to it.)
- **A `!!! template` block is the copy-to-clipboard one.** It is an ordinary
  admonition whose type happens to be `template`, and that is the whole trick:
  no new syntax, so it round-trips like any other block. It renders as a panel
  with a copy button, in the public reader **and** in Odoo's editor. Write email
  templates, snippets and boilerplate in it — see the section below.
- **The front matter is never hand-written.** The header is composed from the `title`,
  `metadata` and `front_matter` arguments. Pasting a `---` block works, but the arguments
  win, and a key the documentation does not manage fails the whole call.

## Writing something the reader can copy

`!!! template "Subject"` is how a page offers text to be taken away — an email
template, a snippet, boilerplate someone will paste elsewhere:

```
!!! template "Introducing Dyncloud to a customer"
    Hello **{{name}}**,

    I am writing to introduce [Dyncloud](https://dyncloud.es):

    - a new environment in 24 hours
    - daily backups

    Kind regards,
```

Four things decide how you write one:

- **The body is Markdown**, so bold, italics, links, inline code and lists all
  work and all survive the paste. This is the whole reason it is not a code
  fence: a fence is literal text and formats nothing.
- **The copy takes the BODY, not the title.** The title is the panel's label. If
  the subject line has to end up on the clipboard, write it inside the body.
- **It pastes WITH formatting** into an email client, because the copy carries
  `text/html` and `text/plain` at once. Where the browser cannot do that (an
  insecure context, some mobile in-app browsers) it degrades to plain text.
- **No colours.** Markdown has none, so a template cannot be red or highlighted.
  Emphasis and structure are what you have.

## What does NOT survive

None of these are part of the grammar. They are not errors you will be told about — the
page just gets worse.

| What | What actually happens |
| --- | --- |
| **mkdocs attribute lists**, `[text](url){target=_blank rel=noopener}` | Shown **as literal text** in Odoo's editor, in every language, and the braces stay in the file with an added backslash. Write a plain link. |
| **Footnotes**, `text[^1]` + `[^1]: note` | Comes back as `text\[^1\]`. Never renders. Put the note in the prose or in a collapsible. |
| **Abbreviations**, `*[HTML]: HyperText…` | Comes back as `\*\[HTML\]:…`, a visible line of noise. |
| **Task lists**, `- [ ] pending` | Comes back as `- \[ \] pending`. Never renders as a checkbox. Use a plain list. |
| **A bare `<tag>` in prose**, e.g. `Use <environment> here` | **The save is REFUSED** — the whole page, with an error. Wrap it: `` `<environment>` ``. This is deliberate: refusing beats losing the author's text silently. |
| **An `<svg>` pasted into the Markdown** | Emptied the next time the page is edited. A diagram is a **`.svg` file** uploaded as an asset — `references/mcp/workflows/files.md`. |
| **An Odoo URL as a link between pages** | It breaks the document for every reader outside Odoo. Relative `.md` path, always — `references/mcp/workflows/authoring.md` §7. |

The rule behind the table: **if a construct is not in the first table, it does not exist.**
Something the grammar cannot express belongs in a fenced code block, in a file, or in plain
prose — never in invented syntax that happens to render somewhere else.

## What gets rewritten without asking you

Not damage, and not worth fighting: the round trip normalises these on the first save, and
then they stay put. Do not "fix" them back, and do not report them as a defect.

- Table separators are rewritten to `| --- | --- |`, whatever alignment padding you used.
- `_italic_` becomes `*italic*`, and `__bold__` becomes `**bold**`.
- A literal `_` in prose gains a backslash: `values_prod.yaml` → `values\_prod.yaml`. The
  reader shows it identically; only the file changed.
- List and continuation indentation is normalised.

Because of this, **a page opened in Odoo's editor is often a real diff even when nobody
typed anything**. If you see a document sitting pending publication with a diff of table
separators and backslashes, that is what it is.
