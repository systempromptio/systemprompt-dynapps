# Authoring a curated article

Everything below came out of curating the **first** one. It is here so the second does not
repeat it.

A curated article is not a tidied-up copy of an old one. It is **the current version**: the
old article is a *source*, and where the sources disagree the decided one wins. Say what
changed and why, so the next reader can tell doctrine from history.

## The syntax nobody had written down

Native Markdown authored through `docs_create` / `docs_update` goes through **the same
renderer as the mkdocs import**. Verified by writing it and reading the stored HTML back.
So a hand-written article can look exactly like the Dyncloud corpus, and there is no reason
to hand-write HTML.

| Write this | You get |
| --- | --- |
| `!!! warning "Title"` + 4-space indented body | `<div class="admonition warning">` — a coloured callout |
| ` ```yaml ` fenced block | `<div class="codehilite">` with syntax highlighting |
| `[text](/knowledge/article/874)` | a real internal link between articles |
| `## Heading` | `<h2 id="heading">` — anchors for free, so `[jump](#heading)` works |

Types available: `note`, `info`, `tip`, `question`, `success`, `warning`, `failure`,
`danger`, `quote`, `example`, `bug`, `abstract`.

Use them for **load-bearing** statements, not decoration:

- `info` — the one paragraph somebody should retain if they read nothing else.
- `warning` — the mistake that costs rework.
- `danger` — the mistake that costs credibility with a customer.
- `failure` — what contradicts older documentation that is still live. Name the article and
  link it, so a reader who found the old one first can tell which wins.
- `question` — a genuine open point. Better than silence and far better than inventing an answer.

## The state is the review gate

Create the article **managed and `draft`**, not unmanaged:

```python
{"canon_managed": True, "canon_state": "draft",
 "canon_territory": "<code>", "canon_owner": <uid>, "canon_truth_source": "decided"}
```

`draft` matches the index exclusion rule, so the article is **queued but not retrievable**
until its owner signs it. That is the machine working for you: nobody reads unsigned doctrine,
and no screenshot-and-hope review process is needed. On signing: `draft` → `in_review` →
`verified`, and it enters the index.

Do **not** create it unmanaged "for now". Unmanaged is always eligible, so it would be served
the moment its markdown is written.

!!! warning "activate_canonicity() refuses draft, and that is correct"
    Its guard exists to stop an *already indexed* article being retired by a careless write.
    A brand-new article has nothing to retire, so write the properties directly — and say so
    in the commit, or somebody will "fix" the guard.

## The property traps

All three fail **silently**: the write returns success and stores nothing.

1. **Bulk writes must be grouped by parent.** Odoo refuses `article_properties` across records
   whose parents differ — the definition is identified by the parent record. Writing one
   territory node and its three children is **two** calls, not one. For a subtree of 143
   articles under 30 parents, that is 30 batches.
2. **A root can never carry properties.** So `docs_create_tree` — which requires a `parent_id`
   — cannot create the tree root either. Create the root with `orm_create`, the rest with
   `docs_create_tree`.
3. **The definition does not cascade.** Only the direct parent counts. Provision it on **every**
   node, not just the ones with children today: the first article created under a childless
   node loses its properties silently.

Always re-read after writing. An `ok: true` here means nothing.

## No owner, no signature

`verified` requires `canon_owner` and `canon_territory`. That is not bureaucracy — it is the
whole point, and it will stop you. When the first tree was built, only one of five territories
had an owner, so only that one could be put under management; the other four exist as
containers and stay unmanaged until somebody owns them.

If you find yourself wanting to set an owner "provisionally so it validates", stop: an
unowned document that claims to be signed is worse than an unsigned one.

## Before signing, check the running system

Documents, skills and memory say what we believe; Odoo says what is happening. The three
cheap checks — resolve every record you name, measure adoption instead of asserting it, and
look for a second value of every price — are in `dynbrain`, under *Before writing doctrine,
check the running system*. Read them there rather than trusting a draft.

Two of the four things that check caught on the first article were **a name that did not
exist** and **a config contradicting itself**. Neither was visible in any document.

## Shape that worked

Not a mandated layout — the per-department layouts are still undefined in
`dynbrain/references/documentation-structures/` — but this held up:

1. An `info` callout with what to retain if you read nothing else.
2. The process or flow, as a table linking to its own sections.
3. One section per step, each with the tool that does it.
4. **Measured data** where a claim could be checked: adoption per month, counts, ids.
5. *What we no longer do*, in a `failure` callout, naming and linking the article it replaces.
6. **Sources**, including which of them was read from the live system and on what date.

Point 4 is what separates doctrine from an opinion, and point 5 is what stops the old article
quietly winning because somebody found it first.
