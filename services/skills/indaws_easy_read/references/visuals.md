# Visuals: step 4b (shape) and the HTML page

Contents: Pick the shape · When a visual earns `html` · Dense tables and colour · Explanation aids ·
Build the page · Shape snippets · Table snippets

Meaning has a shape. Text is a line, so the reader rebuilds the shape in their head. A visual
hands them the shape directly. Pick the visual from the content's shape, never for decoration.

## Pick the shape

`CHK mode` prints `SHAPES=` as a hint. You decide.

| If the content is… | Shape | Visual | Chat fallback (one line inside a finding) |
|---|---|---|---|
| Several themes, reasons or documents | overview | **mind map** (always in `html`) | numbered findings |
| Steps in order, with owners or escalation | process | **flow** | `1 Soporte registra → 2 Consultor analiza → 3 Cliente valida` |
| 3+ dates, phases or deadlines | time | **timeline** | `2 mar datos → 16 mar go-live` |
| Parts that should add up to a total | quantity | **breakdown** | `a + b + c = total ≠ stated` |
| Two or more options on the same criteria | choice | **comparison table** | `A: x, rasgo · B: y, rasgo` |
| People or teams that disagree | stances | **positions** | `Por firmar ya: X · Por esperar: Y` |
| The outcome moves with 1-2 inputs (users, price, hours, rate) | what-if | **sensitivity grid** (`table.sx`) | `Con 40 usuarios: 40 × 29,90 × 12 = 14.352 €` |
| Several items share the same 2-5 attributes (modules, risks, suppliers) | inventory | **scorecard** (`table.dense` + status cells) | 🟢 Fabricación · 🟠 Calidad · 🔴 Mantenimiento |
| A grid of magnitudes (hours per module per month) | intensity | **heat table** (`h1`-`h5`) | the largest cell, named |

Besides the shape visual, `html` may open VIS with a **KPI strip**: 2-4 tiles with the figures
the decision rests on. It is not a shape, so it does not use up the one shape visual.

## When a visual earns `html`

A one-line fallback is **not** "choosing a visual": it stays in `chat` or `short`. Choose a real
visual, and so move up to `html`, only when one of these holds:
- **The shape is the finding**: a sum that does not add up, a process the reader must approve or
  follow, 3+ dated milestones with a deadline among them, a decision that depends on a quantity
  the reader can still change (users, edition, hours): the sensitivity grid.
- **The content is an explanation of a process** (verdict "Para entender") and the reader asked to
  understand it: the map and the flow are the answer.
- The input is already `html` by length or by being a bundle.

Otherwise use the fallback line and stay. A 150-word email with two options does not need a page.
`html` always has the mind map plus **at most one** shape visual: the one the decision turns on.
An optional KPI strip and the small tables inside L2 findings do not count toward that cap.

Rules for every visual:
- Every number, date and quote comes from the input or is an `a + b = c` expression. VIS is
  checked like the rest. Bar widths (`style="--w:…"`) and section numbers are styling, not facts.
- Labels sit on the visual itself. No legends to decode.
- A process written in prose is the truth. If a table in the source contradicts the prose, draw
  the flow from the prose and raise `⚠ Contradicción` for the table.

## Dense tables and colour

A table carries the most information per square centimetre, if the eye can scan it. Colour is the
scan path: it tells the eye where to stop. These rules keep it honest and readable.

**Colour has one meaning per page, and never stands alone.** Every coloured cell also carries a
symbol, a word or the number itself, so it still reads in print, in grey and for colour-blind readers.

| Colour | Class | Means | Always with |
|---|---|---|---|
| Green | `c-ok`, `g1`-`g3` | fine, checked, better than the base | `✓` or a number |
| Red | `c-bad`, `b1`-`b3` | wrong, missing, worse than the base | `✗` or a number |
| Orange | `c-warn` | depends on a condition | `⚠` |
| Grey | `c-open` | not checked, unknown | `?` |
| Violet scale | `h1`-`h5`, `db` | size only, no judgement | the number |

Rules:
- **Pick the table by the question.** Status per item → scorecard. Size per item → in-cell bar
  (`td.db`, `--w` = share of the largest value). Size across two axes → heat (`h1` smallest to
  `h5` largest, five steps at most). What happens if → sensitivity grid. Never mix two scales in
  one table.
- **One highlight per table.** The `base` cell in a grid, or one `tr.hl` row: the reader's case,
  the proposal, or the row the finding is about.
- **Shape of a dense table:** at most 7 columns and 12 rows in VIS (more goes to L3 or out).
  Sort rows by what the decision turns on, never alphabetically. Numbers right-aligned
  (`class="n"`), each with its unit. A total goes in `<tfoot>` as an expression.
- **Header cells state the dimension and the unit** ("Custom 29,90 €"), so body cells stay short.
- **In L2, a 3-6 row table replaces a paragraph** when a finding lists items with the same
  attributes. It is shorter and the gate reads it the same way.
- No legend to decode. If a scale needs one, write it in words in the lead: "verde, más barato;
  rojo, más caro".

### The sensitivity grid (`table.sx`)

Its cells are the one place where computed numbers appear without `a + b = c`. Instead, the gate
(G-SX) recomputes every cell from its row and column headers:
- `data-f` is the formula in `r` (row header) and `c` (column header): `r*c*12`, `r*(1+c/100)`,
  `r-c`. Constants come from the input, or are the unit conversions 12, 52, 100 or 1000.
- The first number in each header is its value. A header from the input is written as it is. A
  value you chose to explore (50 users, +10 %) gets `class="hyp"` and the caption says
  `supuesto` and why. Assumed values never appear elsewhere on the page.
- `class="base"` marks the case on the table (the proposal, today). `data-good="low"` (costs,
  hours) or `"high"` (margin, savings) says which way is better. `g1`-`g3` and `b1`-`b3` tint
  cells by how far they are from the base. The gate fails a cell tinted better when it is worse.
- The caption under the grid says the formula in words: "Usuarios × precio por usuario y mes ×
  12 meses". Keep it to 3-5 rows and 2-4 columns: the reader must see the base and both edges.
- One axis is enough: a single result column with an empty or text-only header uses only `r`.

### KPI tiles

2-4 tiles, each: big figure (`.v`), what it is (`.k`), optional context (`.d`). The big figure
comes from the input, or is the result of the expression written in that tile's `.d` line, which
the gate checks. Tile accents: `k-ok`, `k-bad`, `k-warn`. Never a tile for a figure the decision
does not turn on.

## Explanation aids

Use them in L2 when the audience is not the domain expert ("para todos", "para el cliente",
"explícamelo como si…"). Skip them for an expert reader: they slow experts down.

| Aid | Form | Rule |
|---|---|---|
| Glossary | `<abbr title="definición">SLA</abbr>` in HTML; `SLA (acuerdo de nivel de servicio)` once in chat | First use only |
| Worked example | `Ejemplo:` one concrete case | Numbers from the input only |
| Analogy | `Es como…` | At most one, marked as an analogy, never for figures |
| Before / after | `Hoy:` / `Con el cambio:` | Only when the input proposes a change |

## Build the page

1. Copy the structure of `assets/example-body.html` (an invented case, not an eval case). It has
   every block in order, and it passes the gate against `assets/example-input.txt`:
   - hero = L1: label line first, then an `h1` that states the main finding with one
     `<span class="accent">` word, the gist, the `facts` (Te piden, Plazo), the `verdict`, the warnings;
   - VIS: an optional KPI strip, the mind map, then at most one shape visual from the snippets below;
   - L2 (`<details>` per finding, same wording as the branches), L3 table, INJ, LEFTOUT, footer.
   For tables, also read `assets/example-tables-body.html` (input `example-tables-input.txt`):
   KPI strip, sensitivity grid, and a scorecard inside an L2 finding. It passes the gate too.
2. Write it to `<RUN>/body.html`, then run
   `CHK build --body <RUN>/body.html --out <RUN>/draft.html --title "<short title>" --lang <es|en>`.
   Add `--plain` for "sin marca". Never edit the CSS.
3. HTML-escape all input text (`&`, `<`, `>`, `"`). No `<script>`, no `on…=` attributes. Brand
   text: Spanish with full accents, no em dashes, "Dynapps" with a capital D.

Mind map rules (the gate checks the first two):
- 2-5 branches. Each `br-h` is plain text and a statement of 4+ words, never a topic.
- Root = the decision or topic, with the verdict in `<small>`.
- Branch class = the worst status among its leaves: `bad` (✗), `open` (?), `ok` (✓). At most 3
  leaves per branch, each with a status pill.

If `sensitive`, end the label with `· sensible` and add `<span class="sens">Sensible: no publicar</span>`.

## Shape snippets

```html
<!-- flow -->
<ol class="flow">
  <li><span class="who">Soporte</span><br>Registra la incidencia<span class="when">día 1</span></li>
  <li><span class="who">Consultor</span><br>Analiza y propone</li>
  <li><span class="who">Cliente</span><br>Valida la solución</li>
</ol>

<!-- timeline: class="due" marks the deadline that concerns the reader -->
<ol class="tl">
  <li><span class="d">2 de marzo</span><span class="e">el cliente entrega los datos</span></li>
  <li class="due"><span class="d">16 de marzo</span><span class="e">go-live, si se cumple lo anterior</span></li>
</ol>

<!-- breakdown: --w is the bar length relative to the largest value -->
<div class="bd">
  <div class="row"><span>Partida A</span><span class="bar" style="--w:40%"></span><span class="v">a €</span></div>
  <div class="row total"><span>Suma</span><span class="bar" style="--w:100%"></span><span class="v">a + b + c = total €</span></div>
  <div class="row bad"><span>El resumen dice</span><span class="bar" style="--w:90%"></span><span class="v">stated €</span></div>
</div>

<!-- comparison: options as columns, criteria as rows -->
<div class="tbl"><table><thead><tr><th></th><th>A</th><th>B</th></tr></thead><tbody>
  <tr><td>Criterio</td><td>valor A</td><td>valor B</td></tr></tbody></table></div>

<!-- positions -->
<div class="pos">
  <div class="p"><span class="side">Firmar ya</span><b>Equipo o persona</b>Su argumento en una frase.</div>
  <div class="p"><span class="side">Esperar</span><b>Equipo o persona</b>Su argumento en una frase.</div>
</div>
```

## Table snippets

```html
<!-- KPI strip: first thing in VIS -->
<div class="kpis">
  <div class="kpi"><span class="v">40</span><span class="k">usuarios (hoy 25)</span></div>
  <div class="kpi"><span class="v">14.352 €</span><span class="k">licencia anual</span><span class="d">40 × 29,90 × 12 = 14.352 €</span></div>
  <div class="kpi k-ok"><span class="v">12.700 €</span><span class="k">implantación, cuadra</span><span class="d">6.700 + 2.900 + 3.100 = 12.700 €</span></div>
</div>

<!-- sensitivity grid: the gate recomputes every td from data-f -->
<div class="tbl"><table class="sx" data-f="r*c*12" data-good="low">
<caption>Usuarios × precio por usuario y mes × 12 meses. * supuesto: 50 usuarios, por si siguen creciendo.</caption>
<thead><tr><th>usuarios \ edición</th><th>Standard 19,90 €</th><th>Custom 29,90 €</th></tr></thead>
<tbody>
<tr><th>25 usuarios</th><td class="g3">5.970 €</td><td class="g2">8.970 €</td></tr>
<tr><th>40 usuarios</th><td class="g2">9.552 €</td><td class="base">14.352 €</td></tr>
<tr><th class="hyp">50 usuarios</th><td class="g1">11.940 €</td><td class="b2">17.940 €</td></tr>
</tbody></table></div>

<!-- scorecard with in-cell bars and a total -->
<div class="tbl"><table class="dense"><thead><tr><th>Bloque</th><th class="n">Importe</th><th>Experiencia</th></tr></thead><tbody>
<tr><td>Fabricación</td><td class="db" style="--w:100%">6.700 €</td><td class="c-ok">✓ validado en dos clientes</td></tr>
<tr><td>Calidad</td><td class="db" style="--w:43%">2.900 €</td><td class="c-warn">⚠ depende del histórico</td></tr>
<tr><td>Mantenimiento</td><td class="db" style="--w:46%">3.100 €</td><td class="c-bad">✗ nunca implantado aquí</td></tr>
</tbody><tfoot><tr><td>Total</td><td class="n">6.700 + 2.900 + 3.100 = 12.700 €</td><td></td></tr></tfoot></table></div>

<!-- heat table: h1 smallest … h5 largest, the number stays in the cell -->
<div class="tbl"><table class="dense"><thead><tr><th>Módulo</th><th class="n">Oct</th><th class="n">Nov</th></tr></thead><tbody>
<tr><td>Ventas</td><td class="n h2">12 h</td><td class="n h5">40 h</td></tr></tbody></table></div>
```

The L3 claim table needs no classes: a row with a `✗` pill is tinted red by the shell, so the
eye finds the contradictions first.

The L3 status cell starts with its symbol: `✓ coherente`, `✓ comprobado: <URL>`, `✗ contradicho`,
`? sin comprobar`, `? preguntar a <rol>`. ✓ and ✗ count in N.
