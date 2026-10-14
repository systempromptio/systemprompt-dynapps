---
name: indaws-llm-wiki-builder
description: >
  Construye y mantiene un LLM-wiki (patron de Karpathy): base de conocimiento markdown que el
  agente escribe e interconecta incrementalmente, en vez de re-derivar la respuesta por RAG en
  cada consulta. Cubre bootstrap (estructura + schema CLAUDE.md del wiki) y las tres operaciones
  — ingest, query, lint — con scripts deterministas y busqueda BM25, indice vectorial y qmd. Usala
  cuando alguien quiera acumular conocimiento sobre un tema con el tiempo, aunque no diga "wiki".
  Triggers: "monta un wiki", "LLM wiki", "base de conocimiento personal", "segundo cerebro",
  "vault de Obsidian", "voy leyendo papers y quiero organizarlo", "ingesta este articulo en el
  wiki", "documenta este libro capitulo a capitulo", "wiki interno del equipo con las
  transcripciones", "que sabemos ya sobre X", "build a personal knowledge base", "ingest this
  paper", "lint the wiki". NOT memoria de sesion (→ CLAUDE.md) · NOT conocimiento de proyecto
  Odoo de cliente (→ odoo-context-briefing / .ai-memory) · NOT el resumen de un documento.
version: 1.5.0
effort: medium
model: opus
allowed-tools: [Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion, WebSearch, WebFetch, Agent]
---

# LLM Wiki Builder

A knowledge base where **the LLM writes and the human curates**: la persona aporta fuentes y pregunta; tú resumes,
cruzas, archivas y llevas la contabilidad — el trabajo que hace que los humanos abandonen sus wikis. Patrón y upstream:
[source-gist.md](references/source-gist.md) · [upstream-sources.md](references/upstream-sources.md).

Su afirmación no obvia: RAG **re-deriva** el conocimiento en cada pregunta; un wiki lo **compila una vez y lo
mantiene** — referencias cruzadas ya puestas, contradicciones ya marcadas, síntesis ya al día. Compone. Por eso un wiki
que sólo acumula páginas-resumen es un RAG *peor*: un salto más y ninguna síntesis. **El valor está en el recableado.**

## Three layers
| Layer | Owner | Rule |
|---|---|---|
| `raw/` sources | the human | **Immutable.** You read, never edit. Source of truth. (Code is cited by SHA, not copied.) |
| `wiki/` pages | you | You create, update, link, keep consistent. The human only reads. |
| `CLAUDE.md` schema | both, co-evolved | The wiki's constitution: layout, conventions, workflows. |

El schema sostiene todo: convierte una sesión futura en un mantenedor disciplinado, así que **lo que
aprendas sobre este wiki se escribe ahí**. Layout (`wiki_scaffold.py`; `code` se desvía y lo dice):

```
<wiki-root>/   # repo git y vault de Obsidian a la vez: historial y grafo gratis
  CLAUDE.md    # el schema — se lee primero, cada sesión
  index.md     # catálogo: cada página, una línea · log.md — cronología
  open-questions.md   # huecos, contradicciones pendientes, qué buscar
  raw/         # fuentes inmutables (+ raw/assets/)
  output/      # artefactos renderizados (decks, PDF, gráficas) — NO son páginas de wiki
  wiki/        # overview.md · sources/ · entities/ · concepts/ · syntheses/
```
Link with `[[wikilinks]]`: Obsidian and the lint script both follow them.

## Routing — detect the mode, then read
| Signal | Mode | Read |
|---|---|---|
| no wiki yet, or a topic the user wants to accumulate | **Bootstrap** | § below + [profiles.md](references/profiles.md) |
| a file/URL/paste arrives: "add this", "ingest", "file this" | **Ingest** | [operations.md](references/operations.md) § Ingest |
| a question about the subject matter | **Query** | [operations.md](references/operations.md) § Query |
| "is the wiki ok", cleanup, ≥10 ingests since the last check | **Lint** | [operations.md](references/operations.md) § Lint |
| writing or fixing a page | — | [page-formats.md](references/page-formats.md) |
| writing the wiki's own `CLAUDE.md` | — | [schema-authoring.md](references/schema-authoring.md) |
| finding things in a grown wiki, or indexing raw sources before writing | — | [search-qmd.md](references/search-qmd.md) |
| **the sources are a source tree** (Odoo, client modules) | **Code** | [code-profile.md](references/code-profile.md) — read before starting |

Si ya existe un wiki, **lee su `CLAUDE.md` primero**: ese schema manda sobre esta skill — se escribió para ese wiki y lo co-firmó la persona.

## Bootstrap
La entrevista importa más que el scaffold (`AskUserQuestion` sólo para lo que no puedas inferir): **(1)** tema y para
qué lo quiere · **(2)** perfil de dominio → [profiles.md](references/profiles.md), que fija la taxonomía ·
**(3)** fuentes: tipo, volumen, cómo llegan, si las imágenes importan · **(4)** supervisión: fuente-a-fuente con
discusión (mejores wikis) vs batch, y dónde vive (¿repo git? ¿vault de Obsidian?).

Luego `python3 scripts/wiki_scaffold.py <root> --title "…" --profile research`, y **reescribe el `CLAUDE.md`
generado** para este dominio: un schema genérico da un wiki genérico. Cierra **ingestando la primera fuente de punta a
punta** — es la única forma de descubrir que la taxonomía está mal cuando cambiarla todavía es barato.

## The three operations
**Ingest.** La fuente aterriza en `raw/`, se lee, se comentan los hallazgos *antes* de escribir (una pregunta ahora
vale más que reescribir 12 páginas después), se escribe la página de fuente y luego se **recablea**: cada
entidad/concepto/síntesis que toca, páginas que ahora se ganan, contradicciones marcadas, enlaces en ambos sentidos,
`index.md` + `log.md`. Una fuente sustancial toca 10-15 páginas; si la tuya tocó 2, o la fuente era pobre —dilo— o te
saltaste el trabajo.

**Query.** `index.md` primero, luego las 3-8 páginas que señale; a búsqueda sólo si eso falla. Cita, y
di claramente cuando el wiki no lo sabe. Y después: **¿esta respuesta merece página?** La síntesis de
verdad se muere en el historial del chat — archívala en `wiki/syntheses/` e indéxala.

**Lint.** `wiki_lint.py` para la podredumbre mecánica (~0 tokens) y luego el pase de juicio que un
script no puede: contradicciones, significado caducado, páginas a fusionar, síntesis pobre,
huecos. `open-questions.md` al día y el pase registrado.

## Non-negotiables (and why)
- **Nunca modifiques `raw/`** — es el rastro de auditoría: la fuente gana al wiki, y sólo se puede
  comprobar si nadie la ha tocado.
- **Toda afirmación no obvia lleva cita** `(→ [[s-2026-08-03-slug]])`, o el wiki es prosa infalsable y su premisa
  —fiable porque es trazable— se cae.
- **Las contradicciones se registran, no se sobrescriben**: las dos versiones fechadas, cuál crees y
  por qué. El conocimiento que *cambió* es lo más valioso que hay aquí.
- **Ninguna ingesta está hecha hasta que `index.md` y `log.md` están actualizados** — lo no indexado
  es invisible, así que no pasó.
- **Di lo que no sabes** (`open-questions.md`, `confidence: low`): un wiki que esconde sus puntos
  flacos se usa donde no debe. Y **escribe para la sesión siguiente**, que no recuerda este chat.

## Gotchas
- **Wikilinks resolve by basename**: same filename in two directories = every link to them is
  ambiguous (`wiki_lint.py` errors on duplicate slugs).
- **Parallel subagents must never write `index.md`, `log.md` or shared pages** — concurrent edits to
  one file silently lose data, and shared pages are where agents collide. So batch ingest is
  two-phase: extract in parallel (own source page only), integrate serially.
- **Los tres gotchas del wiki de código** —estructura escrita de memoria, una cita con `@sha` que no *resuelve*, y
  varias versiones como varios wikis de prosa en vez de base+deltas— tienen su detalle en
  [code-profile.md](references/code-profile.md): son los que hunden un wiki de código, se leen antes.
- **Las tres trampas de las fuentes** —un fetch truncado que produce una página confiadamente falsa,
  un PDF abierto a ciegas (`Read` = ~1.7K tokens por página: `pdf_probe.py` antes), y un índice
  caducado que responde en silencio (`qmd update`, `code_query.py … drift`, `wiki_log.py`)— con el
  procedimiento en [operations.md](references/operations.md) y [search-qmd.md](references/search-qmd.md).

## Token discipline and agent routing
Un wiki se toca cientos de veces: lo chapucero se multiplica por esa cifra.

- Mechanical checks are **scripts, never agents** (link graph, index drift, frontmatter, code
  structure). Measured: full Odoo 19 CE+EE inventory = 1.428 modules in 36 s, zero tokens.
- Escalera de recuperación `index.md` → `wiki_search.py` → **índice vectorial local** (`wiki_index.py`+`wiki_vsearch.py`, embeddings delegables a
  otra máquina) o `qmd`; no montes búsqueda con 20 páginas. **Medido**: gana en pasajes de documentos grandes y en consultas que parafrasean (6/6
  al #1); **falla en las inferenciales sin solape léxico (0/4)**, que es justo lo que prometía — números y causas descartadas en
  [search-qmd.md](references/search-qmd.md). **Una fuente (o un subsistema) a la vez**: contexto acotado y humano en el bucle.
- Reparto: la **extracción** en batch es mecánica → subagentes `sonnet`, uno por fuente. **Integración,
  síntesis, juicio de contradicciones y el pase de juicio del lint se quedan en `opus`** — ahí
  equivocarse es caro. Pre-flight de contexto antes de cualquier fan-out.

## Scripts
Scripts deterministas (0 tokens: se ejecutan, no se leen); qué hace cada uno, en [references/scripts.md](references/scripts.md). Siempre:
`wiki_scaffold.py` · `wiki_lint.py` · `wiki_search.py` (BM25) · `wiki_log.py`. Corpus grande o varios wikis: `wiki_index.py` / `wiki_vsearch.py`
(índice vectorial) · `wiki_dupes.py` (qué consolidar). Wikis de código: `code_extract.py`/`code_query.py` · `wiki_cite_check.py` (¿la cita
*resuelve*?) · `wiki_coverage.py` (¿de qué habla ya la prosa?) · `wiki_migration.py`. PDFs: `pdf_probe.py` antes de `pdf_extract.py`. Upstream:
`check_upstream.py`.

## Delegates to
| Skill / reference | For what |
|---|---|
| `references/shared/odoo-source.md` | **Where the Odoo source lives.** The `code` profile needs an addons path; never hardcode `/opt/odoo` — that is one environment's convention |
| `odoo-deep-knowledge` (odoo-knowledge) | Gotchas already catalogued. Link to them; a code wiki re-deriving them wastes the ingest |
| `odoo-version-api-reference` (common) | Documented API diffs between majors — cheaper than a `diff` when the answer is already written |
| `odoo-pilot` / MCP (common) | Runtime verification (`fields_get`, `ir.rule`) when a claim is about an install, not about code. Contract in `references/shared/odoo-connection.md` |

## Consumed by
**Nadie, hoy.** Es una skill de entrada: la invoca la persona, no otra skill. Se dice porque lo
contrario —declarar consumidores que no la citan— es la forma barata de aparentar integración. El día
que alguna delegue aquí, esta línea lo dirá y se verá con un `grep`.
