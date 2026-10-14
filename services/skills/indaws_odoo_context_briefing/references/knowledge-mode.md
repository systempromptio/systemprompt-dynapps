# Modo `knowledge` — KB verificada del custom del cliente

El modo `knowledge` produce el **análogo por-cliente de `odoo-deep-knowledge`**: conocimiento profundo
y **verificado contra la fuente real** del custom del cliente. Es el complemento de profundidad al
mapa mecánico de `inventory/`.

- **Scope**: SOLO módulos `custom` (opcionalmente `indaws_core` si el usuario lo pide). NUNCA estándar/oca.
- **Salida**: `.ai-memory/knowledge/` (hermano de `inventory/`, dentro del git del cliente).
- **Coste**: alto (un agente por tanda que lee y verifica). Por eso es **explícito, nunca automático**:
  no lo disparan `full`/`target`/`incremental`. Solo corre si el usuario pide "genera/actualiza knowledge".

## Estructura de salida

```
.ai-memory/knowledge/
├── _index/symptoms.md         # síntoma → módulo → gotcha (GENERADO, no editar a mano)
├── modules/<módulo>/
│   ├── core.md                # propósito, modelos, flujo, depends — anclado a fichero.py:línea
│   └── gotchas.md             # comportamientos ocultos, convenciones, riesgos, bugs — anclado
├── domain/<topic>.md          # conocimiento HUMANO compilado (decisiones, reglas de negocio,
│                              #   acuerdos, "por qué") — cita `raw/…`, NO tiene código que verificar
├── raw/                       # fuentes humanas INMUTABLES (notas, actas, PDFs→md, emails de
│                              #   decisión) — baseline de verdad; el LLM las LEE, nunca las edita
├── log.md                     # audit trail APPEND-ONLY (cada ingest / contradicción / decisión)
└── _bundles/                  # insumos regenerables (gitignored) — NO son conocimiento
```

`INDEX.md` en la raíz de `.ai-memory/` es el punto de entrada único (lo genera `gen_knowledge_index.py`).

## Dos clases de conocimiento (no confundir)

| | Derivado de código (`modules/`) | Humano (`domain/`) |
|---|---|---|
| Fuente de verdad | el código del cliente (`/…/*.py`) | `raw/` (actas, decisiones, reglas de negocio) |
| Verificación | anclas `fichero.py:línea` contra fuente real | cita `raw/<file>`; el raw ES la verdad (no hay código) |
| Cuándo | comportamiento oculto, bugs, convenciones del código | por qué se decidió algo, reglas que NO están en el código |
| Se regenera | al cambiar el código | al ingerir nuevo material en `raw/` |

Regla: si el dato está en el código → `modules/`. Si es una decisión/acuerdo/regla de negocio que **el código no explica** → `domain/`, citando su `raw/`. Nunca inventes `domain/` sin una fuente en `raw/`.

## Pipeline

```
[OPUS principal] Paso K0: resolver repo custom del cliente + lista de módulos custom
   (desde inventory/modules.yaml: type=custom con source_path)
   |
   v
[OPUS principal] Paso K1: build_knowledge_bundle.py
   pre-arma UN bundle compacto por módulo (README + deep/<m>.md + DESARROLLO_LOG +
   mapa de símbolos con nº de línea + manifest + ficheros). SIN LLM.
   → knowledge/_bundles/<m>.bundle.md
   |
   v
[N SONNET subagents · UNO POR TANDA de módulos] Paso K2: autor + verificación
   cada agente lee el/los bundle(s), VERIFICA anclas dudosas contra la fuente real,
   y escribe core.md + gotchas.md. Enrutar por tamaño: report/solo-datos = tanda barata.
   |
   v
[OPUS principal] Paso K3: gen_knowledge_index.py
   regenera _index/symptoms.md + INDEX.md de forma determinista (extrae títulos de gotchas).
```

**Leer 1 bundle en vez de rastrear el árbol** baja de ~20-33 tool calls a ~4 y evita re-leer lo mismo.

### Rendimiento (lección del MVP)

La 1ª mitad del MVP (un agente por módulo + acuse por notificación) fue cara; el tramo final
(bundles + 1 agente para 7 módulos) costó **~5× menos**. Para nuevos repos: **bundles + 1 agente por
tanda + enrutar por tamaño**. Antes de un fan-out grande, correr el pre-flight de contexto (ver CLAUDE.md).

## Contrato de formato (OBLIGATORIO para que el índice sea determinista)

Cada gotcha DEBE empezar con un título en negrita `**<ID>. <título conciso>.**`:

```
**BOMC1. Orden de `sequence` en los mappings de Fiscal Position.** Detalle largo…
```

- `<ID>` = prefijo del módulo + número (`BOMC1`, `RE12`; se admite guion tipo `BR-G1`).
- El **título** va DENTRO de la negrita, en una sola línea. `gen_knowledge_index.py` lo extrae para
  `symptoms.md`. Si la negrita envuelve solo el ID (`**TP1.** texto…`), el índice recorta el primer
  inciso — funciona pero da títulos peores. **Preferir siempre la forma completa.**
- Anclas siempre `fichero.py:línea` o `::método`, verificadas contra la fuente del cliente.

## Hallazgos de calibración (por qué la verificación es imprescindible)

1. **README y ai-memory deep suelen estar desfasados** (versión/LOC/deps). Los agentes lo detectan
   leyendo el código y corrigen. La verificación contra fuente NO es opcional.
2. **Se detectan bugs reales, no solo doc**: p.ej. `qty_done` inexistente en v18 enmascarado por tests,
   overrides referenciados pero ausentes, tests rotos, direcciones duplicables en split.
3. **En módulos pequeños los tests valen más que el ai-memory deep** como semilla fiable.

## Ingest de fuentes humanas (`raw/` → `domain/`)

Para conocimiento que NO está en el código (decisiones de proyecto, actas de reunión, reglas de
negocio acordadas con el cliente, criterios de un CTO, PDFs de especificación):

1. **Dropear la fuente en `knowledge/raw/`** tal cual (markdown, o PDF/doc convertido a `.md`).
   Nombrarla con fecha+tema (`raw/decisiones/2026-07-19-iva-agencias.md`). NO se edita después.
2. **Compilar a `domain/<topic>.md`**: el agente lee el/los raw, sintetiza la página de concepto y
   **cita la fuente** con la ruta relativa `raw/…` (para que el lint la valide y sea trazable).
   - **Nueva página vs editar en sitio**: si el tema ya tiene página → editar en sitio y añadir la
     nueva cita; si es un tema nuevo → nueva página. Nunca dupliques páginas del mismo tema.
   - **Contradicción**: si el nuevo raw contradice lo escrito, NO sobreescribas a ciegas: marca
     `⚠CONFLICT` en la página con ambas fuentes citadas y regístralo en `log.md` para que un humano
     decida. El lint lo reporta hasta que se resuelve.
3. **Anotar en `log.md`** (append-only): `## [YYYY-MM-DD] ingest | <tema>` + qué raw entró y qué
   página se creó/actualizó.
4. **Regenerar índice**: `gen_knowledge_index.py` (mete las páginas domain y los contadores en INDEX).

## `log.md` — audit trail append-only

Un solo fichero `knowledge/log.md`, **solo se añade** (nunca se reescribe). Cabeceras grepables:

```
## [2026-07-19] ingest | política de IVA de agencias
Fuente: raw/decisiones/2026-07-19-iva-agencias.md → domain/iva-agencias.md (nueva).
## [2026-07-20] conflict | margen refacturación
raw/…/acta-B.md contradice domain/margenes.md (fuente previa raw/…/acta-A.md). ⚠CONFLICT, pendiente CTO.
```

Sirve para: reconstruir cómo evolucionó el conocimiento, y auditar de dónde salió cada afirmación.

## `lint` — chequeo de consistencia (determinista, sin LLM)

```
gen_knowledge_index.py --ai-memory <path/.ai-memory> --lint     # exit 1 si hay issues
```
Comprueba: citas `raw/…` rotas en páginas `domain/`, páginas `domain/` huérfanas (sin fuente
trazable), marcadores `⚠CONFLICT` sin resolver, ausencia de `log.md` habiendo `domain/`, y material
en `raw/` sin compilar a ninguna página. Correr tras cada ingest y antes de dar la KB por buena.

## Regeneración parcial

- Un módulo/tanda: `build_knowledge_bundle.py --modules <m1> <m2>` → re-autor solo esos → `gen_knowledge_index.py`.
- Solo el índice tras editar gotchas: `gen_knowledge_index.py --ai-memory <path> --symptoms-only`.
