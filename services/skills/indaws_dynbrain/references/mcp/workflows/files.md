# Workflows — the FILES of a documentation

A documentation is prose plus the diagram, the PDF and the spreadsheet the prose is about.
Three tools cover that half of the corpus — `docs_asset_list`, `docs_asset_link`
(`../read-tools.md`) and `docs_asset_upload` (`../write-tools.md`) — and the one thing they
have in common is what they deliberately do **not** do: **the bytes never travel through
this channel**, in either direction. Every recipe here is therefore two halves: an MCP call
that hands you a URL, and somebody moving the file over that URL.

**You have no tool that uploads or downloads a file.** Not a gap you can work around with a
clever call — moving the bytes is your shell's job, or the user's browser's. Which of the
two depends on the environment you are running in, and picking wrong is how a session
stalls asking somebody to run a command they have no terminal for. §2 and §3 each start by
choosing.

> **The URLs expire in 15 minutes, and an upload URL is single use.** `expires_minutes`
> says so in every answer. If the person takes longer, ask for another one — retrying with
> the old one gets "Invalid or expired upload link." and no amount of retrying fixes it.

---

## 1. What files does this document have?

The call that has to come first, because until it answers you cannot name a file, size it,
download it or tell whether Git has it.

**`docs_get` will not tell you.** It returns the document's text and nothing about its
files, so a page whose diagram matters comes back as `![Topology](arquitectura.svg)` — a
relative name and no id, no size, no way to fetch it. The reference in the prose is the
only clue there is one. So when a document mentions a diagram, a PDF or a spreadsheet and
what it shows is part of the answer, **ask**: the file is one call away and you will
otherwise describe a picture you never opened.

```text
docs_search { query: "envoy gateway" }        # or docs_get, or an id you already hold
docs_asset_list { article_id: 3852 }
  → files: [ { attachment_id: 369846, name: "envoy-flow.svg", size_bytes: 18422,
               in_repository: true,  path: "docs/network/envoy-flow.svg", pending_publication: false },
             { attachment_id: 369901, name: "manual.pdf",     size_bytes: 2411903,
               in_repository: false, path: "",                pending_publication: true } ]
```

Narrate the pair, never one flag: the first file is in the repository, **the second is not
and travels with the documentation's next publication**. A row that is `in_repository: false`
**and** `pending_publication: false` is one nothing will ever publish — right and complete on
a native document, a file that fell out of the flow on a Git one. And mind the two meanings of `in_repository` — the
top-level one is the document's, the one inside a `files[]` row is the file's
(`../read-tools.md` → `docs_asset_list`).

---

## 2. Download a file — and read it

```text
docs_asset_link { attachment_id: 369901 }
  → { name: "manual.pdf", size_bytes: 2411903,
      download_url: "https://…/download/<token>", expires_minutes: 15 }
```

Then move the bytes. **Choose by environment:**

**With a shell (macOS, Linux and Windows alike).** `curl` ships with Windows 10 and 11, so
the same line works everywhere:
```bash
curl -sL -o manual.pdf "<download_url>"
```
In **Windows PowerShell** `curl` is an alias for `Invoke-WebRequest`, where `-o` means
something else — call `curl.exe` explicitly, or use the native form:
```powershell
Invoke-WebRequest -Uri "<download_url>" -OutFile manual.pdf
```

**When you need to READ what is inside** — summarise the PDF, look at the screenshot,
check what the diagram actually says — download it to disk **and then open it with the
file-reading tool**, which understands images and PDFs. Never pipe the bytes into the
conversation: `cat`ting a PDF or base64-ing an image fills the context with noise and
tells you nothing.
```text
curl -sL -o /tmp/manual.pdf "<download_url>"     # then read /tmp/manual.pdf with the Read tool
```

**Without a shell (Cowork, and any environment where you cannot run commands): hand the
URL to the person.** It is a plain download link served with their own permissions —
opening it saves the file. Say what it is and that it expires:

> "Here is the manual (2.4 MB): `<download_url>`. The link expires in 15 minutes; if it runs
> out on you, I will generate another one."

Note that this path gets the file to the **person**, not to you: you will not have read it.
Say which of the two happened rather than implying you looked at something you did not.

---

## 3. Attach a file to a document

Two calls with the upload in between. **The first call refuses an executable on the name
alone**, before anybody spends time uploading 25 MB.

```text
docs_metadata { article_id: 3852 }                 # origin FIRST: native / managed / mirror
docs_asset_upload { article_id: 3852, filename: "network-diagram.svg" }
  → { attached: false, upload_url: "https://…/upload/<token>", filename: "network-diagram.svg",
      max_bytes: 26214400, expires_minutes: 15 }
```

**With a shell**, do it yourself:
```bash
curl -s -F "file=@network-diagram.svg" "<upload_url>"
#  -> {"ok": true, "attachment_id": 369846, "bytes": 18422}
```
PowerShell alternative:
```powershell
Invoke-WebRequest -Uri "<upload_url>" -Method Post -Form @{ file = Get-Item .\network-diagram.svg }
```

**Without a shell, or in Cowork: give the URL to the person.** Opening it in a browser
serves a small upload form — pick the file, submit, and it goes **straight into Odoo**;
nothing passes through the conversation. Then ask them for the `attachment_id` the page
answers with:

> "Open this in your browser and upload the diagram there: `<upload_url>`. It is a
> single-use link and it expires in 15 minutes. When you upload it you get back an
> `attachment_id`; pass it to me and I will attach it to the page."

Then the second call, and the file becomes a file of the document:
```text
docs_asset_upload { article_id: 3852, attachment_id: 369846 }
  → { attached: true, file: { name: "network-diagram.svg", pending_publication: true, … },
      variant: { lang: "es", … }, message: "… pending publication … the page must reference it …" }
```

**Now make it visible.** The upload does not put the file in the prose, and a file nobody
links is a file nobody sees:
```text
docs_get    { article_id: 3852 }                 # the current body — docs_update REPLACES it
docs_update { article_id: 3852,
              markdown: "<that body, with ![Network diagram](network-diagram.svg) added>" }
```
**Relative, by name — never the attachment id.** The file is committed next to the page, so
`network-diagram.svg` resolves in Odoo, in GitLab's file viewer and in any Markdown editor;
a `/web/content/369846` works in exactly one of them. Use the `name` the answer reports,
which is the sanitised one, not necessarily the one you sent.

**And read the `message` and `pending_publication` before you say anything.** On a
**managed** document the file is in Odoo and in no repository until the documentation's next
publication carries it — same journey as an edit (`pending.md` §2). On a **native** document
there is nothing to publish and saying "done" is the truth. The `message` is composed for the
server you are on, so read it out rather than substituting a step of your own.

---

## 4. A new version of a file that is already published

**Uploading it again under the same name does not replace it.** New bytes under a name the
repository already holds are published *beside* the old file, numbered
(`network-diagram-2.svg`), and the page keeps linking the old one — deliberately, because
the alternative is overwriting somebody else's file inside a commit about something else.
Identical bytes are the one exception: they are recognised as the same file and nothing is
committed twice.

So do it in the open, and say what you are doing:

```text
docs_asset_upload { article_id: 3852, filename: "network-diagram-v2.svg" }   # … then the id
docs_update { article_id: 3852, markdown: "…![Network diagram](network-diagram-v2.svg)…" }
```

> "I have uploaded the new diagram as `network-diagram-v2.svg` and changed the page to point
> at it. The old one is still in the repository; if you want it gone, it has to be deleted
> there."

Retiring the old file is a repository job, not one of these tools': the publication only
removes a file when the page that linked it is retired, it sat in that page's own folder,
and no surviving document points at it.

---

## 5. A diagram goes in a FILE, never inline in the Markdown

When you write a page with a diagram, **upload it as a `.svg` file and link it by name**
(§3). Do not paste an `<svg>…</svg>` into the Markdown, however tempting it is when a tool
or a model hands you the markup.

Both render today, so this is not a rule about what works — it is about where each one
breaks:

| | inline `<svg>` | `.svg` file |
| --- | --- | --- |
| Reader | works, through an extraction with known limits | works |
| **Edited inside Odoo** | **empties out — the page shows a hole** | works |
| Second language | the whole drawing, copied into both pages | one file, linked twice |
| Downloadable, versioned, diffable | no | yes |

The middle row is the one that costs a page. The document body is sanitised HTML, which
keeps the `<svg>` wrapper and deletes the drawing inside it, so the diagram survives only
because the IMPORT extracts it into an attachment. A diagram written or changed **in
Odoo** never went through an import: nothing extracts it, the sanitiser empties it, and
the page renders a hole until the change reaches Git and comes back.

If you meet a page that already has one, say so plainly and offer to move the drawing into
a file: `docs_asset_upload` the `.svg`, then `docs_update` the body with
`![Title](diagram.svg)` in place of the markup. It is the same content and it stops being
fragile.

## 6. An SVG file that does not show

If a diagram uploads fine, publishes fine, and simply does not render: it is almost
certainly an SVG carrying something runnable — a script tag, an `onload=`, a
`javascript:` URL, a `<foreignObject>`, a declared entity. Those are stored and published,
but **not labelled as an image**, so nothing renders them inline. Several drawing tools
export handlers without being asked.

Nothing is broken and nothing is lost: the file is in the repository and downloadable. Say
what it is and what fixes it — export the diagram as a plain SVG (or a PNG) and upload that
version. Don't re-upload the same bytes hoping for a different label.
