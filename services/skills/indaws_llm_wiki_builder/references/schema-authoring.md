# Authoring the wiki's schema (its `CLAUDE.md`)

The schema is the highest-leverage file in the wiki. It is what a session reads before touching
anything, and it is the difference between a disciplined maintainer and a generic chatbot with
file access. `wiki_scaffold.py` writes a working draft; this document is how you turn that draft
into something worth obeying.

Its readers are: **future you** (a session with zero memory of this conversation), **other
agents** the user may point at the wiki, and **the human**, who will edit it when your workflow
annoys them.

## What belongs in it

1. **What this wiki is for.** Subject, purpose, and what the human wants to do with it. Without
   this, every judgement call downstream is a coin flip.
2. **The layout**, as it actually is — including deviations from the default. If you added
   `wiki/timeline/`, the schema is where that becomes real.
3. **Page taxonomy for this domain**: which page types exist here, what earns a page, what
   doesn't. This is where [profiles.md](profiles.md) gets specialised into local law.
4. **Conventions**: naming, frontmatter fields, citation syntax, contradiction handling. Copy the
   decisions from [page-formats.md](page-formats.md) that apply, and record the ones you changed.
5. **Workflows**: what ingest / query / lint mean *here*, concretely. Include the local details —
   which pages a source usually touches, how supervised the user wants ingest to be, output
   formats they like.
6. **Retrieval**: index-first, and when to escalate to search. Name the tool if one is installed.
7. **Log and index rules**: format, and the rule that an operation isn't finished until both are
   updated.
8. **Scale notes**: current size, and what to do when it grows (split a directory, promote a
   concept cluster, introduce search).
9. **Local gotchas**: things you learned the hard way. "The 2019 reports use the old fiscal
   calendar." "Interview transcripts have speaker labels that break the summariser." This section
   is what makes a schema *this wiki's* schema.

## What to leave out

- The pattern's rationale — that's [source-gist.md](source-gist.md), don't paraphrase it.
- Long instructions on how to write markdown. The reader is an LLM.
- Aspirations nobody follows. A schema with rules the user overrides every session is worse than
  a short honest one: it trains the next session to ignore the file.

## Style

Short, declarative, specific. State the rule and *why* it exists in the same breath — a rule with
a reason survives contact with an unforeseen situation, a bare `ALWAYS` does not. Aim for one
screen of content plus the gotchas; a 500-line schema won't be read carefully.

Write in the user's working language.

## Co-evolution

The schema is a living file, and keeping it current is part of every operation:

- When the user corrects how you did something, that correction goes into the schema. Otherwise
  the next session repeats the mistake and the user has to correct it again — the fastest way to
  lose their trust in the whole setup.
- When you invent a convention mid-ingest, write it down before the session ends.
- When the wiki outgrows a rule (a directory with 200 files, a taxonomy that stopped fitting),
  propose the change to the user, then amend the schema.
- Lint passes are a good moment to ask: does the schema still describe what we actually do?

Keep it under version control with the rest of the wiki, so its history shows how the wiki's
understanding of itself evolved.

## Template

`scripts/templates/schema-CLAUDE.md` — the scaffold copies it with placeholders filled. Treat it
as a checklist of sections to make specific, not as a finished file. A schema that still reads
like the template is a schema nobody adapted.
