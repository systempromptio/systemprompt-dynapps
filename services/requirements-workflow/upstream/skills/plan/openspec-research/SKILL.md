---
name: openspec-research
description: Empirical research for an OpenSpec discovery/design gap that committed sources cannot settle. Use when an artifact needs a fact you must go and check, or when the user asks for a research/investigation.
---

# OpenSpec Research

A research turns an empirical check — a browser session, a network capture, an API probe, an
instance inspection, a large code sweep — into a **citable source**: a `report.md` under the change's
`research/` folder that later artifacts anchor to like any other source. A subagent runs it; you
decide whether it is warranted.

Applies to `discovery` and `design` only, inside an existing change folder.

## Is it warranted

All three must hold:

- the answer needs a run or a traversal whose result cannot be cheaply reconstructed by reading;
- the fact materially changes an artifact — name the consumer (a Gap Analysis Current State cell, a
  requirement disposition, a decision's grounding) or drop it;
- the result is worth carrying to a reviewer or a later session.

Not a research:

- the answer is already in a spec, a signed requirement, or a meeting note;
- two files need reading, or one grep settles it;
- you want to re-confirm your own understanding of something you already read;
- the question is how we will build it, not how things behave today;
- it is a project document to fetch (project-context) or an implementation bug
  ([systematic-debugging](mdc:.cursor/skills/test/systematic-debugging/SKILL.md)).

## If it is warranted

You MUST name the gap and offer a research instead of quietly recording "not verified"; you MUST NOT
start one without the user's yes (a direct request is consent).

Then read
[running-a-research](mdc:.cursor/skills/plan/openspec-research/references/running-a-research.md) —
scoping, launch packet, registration, citation — and launch the `openspec-research` subagent.
