# Cómo se extiende / mejora este KB

> El *cómo mecánico* (IDs, patrón de ficheros, gates) está en `SKILL.md` (ALWAYS 1-5).
> Este doc es el **método**: cómo decidir QUÉ documentar y cómo hacerlo sin alucinar.

## Principio: documentamos CAPACIDADES verificadas, no recordamos features

Hay dos ejes de conocimiento, y se documentan distinto:

| Eje | Pregunta que responde | Dónde vive | Lo escribe… |
|-----|----------------------|------------|-------------|
| **Implementación / debugging** | "por qué Odoo hace X / dónde se rompe" | `gotchas.md`, `integrations.md` | leer fuente al depurar/diseñar |
| **Capacidad (consultor)** | "¿Odoo PUEDE hacer X? ¿y cuánto cuesta?" | `capabilities.md` (fichero especializado) | el método de abajo |

El eje *capacidad* es el que aporta a una venta/GAP: el cliente pregunta "¿se puede hacer X?" y necesitamos **dos respuestas**: (1) **se puede o no**, y (2) **la complejidad** (nativo / configuración / desarrollo). El valor está en los casos "sí, pero no de forma obvia" — por eso se verifica contra fuente, no de memoria.

## Cosecha desde tickets/incidencias (intake)

Además de las preguntas-cliente, el KB se alimenta de la **operación real**: al resolver un ticket o
incidencia (`ticket-solver`) o una tarea de dev, si la causa raíz es un comportamiento oculto de Odoo
NO documentado, se deja un **candidato** en [`gotcha-candidates.md`](gotcha-candidates.md) (inbox).
Los candidatos NO son gotchas todavía: se curan con el ciclo de abajo (fuente, ID, índices, gate) y
entonces se mueven al `gotchas.md` del dominio. Genérico va a este KB; específico de un cliente va a su
`project-knowledge`.

## El ciclo (question-driven coverage)

1. **Partir de las preguntas del cliente.** Lista las preguntas reales que un cliente haría sobre el dominio ("¿puedo facturar por uso?", "¿hay periodo de prueba?", "¿el cliente puede cambiar de plan solo?"). Son el índice de cobertura: cada pregunta = una fila de `capabilities.md`.

2. **Responder SOLO contra fuente.** Para cada pregunta abre el `.py`/`.xml` del módulo (`odoo_enterprise/<v>/<módulo>`, `odoo<v>/addons/<módulo>`). La respuesta es binaria respecto a la fuente:
   - **sí, nativo** → existe el campo/método/flujo. Ancla `archivo.py::método`.
   - **sí, con configuración** → existe pero requiere flags/datos (di cuáles).
   - **sí, con desarrollo** → no existe pero hay punto de extensión (di cuál: override de `_método`, hook, mixin).
   - **no** → no está y no hay punto de enganche obvio. **"no" es una respuesta válida y útil** — di "no" cuando la fuente no lo soporta; no inventes un camino.
   - **no encontrado en fuente** → si no lo localizas, escríbelo así; NO lo conviertas en "sí" por intuición.

3. **Formato de fila** (en `capabilities.md`):
   `**¿Capacidad?** — VEREDICTO (nativo/config/desarrollo/no) · complejidad (baja/media/alta) · ancla \`archivo.py::método\` · 1 frase de cómo/por qué.`

4. **Anclar a símbolo, no de memoria.** Igual que los gotchas: `archivo.py::método` o `Modelo.campo`. Si afirmas "Odoo hace X en `_método`", ese `_método` DEBE existir en fuente (lo valida `cite_audit.py --methods`).

5. **Actualizar índices.** Tras escribir: añade entradas a `_index/symptoms.md` (las preguntas-capacidad como síntomas) y `_index/cheatsheet.md`.

6. **Pasar los gates.** `scripts/kb_release_check.py` (un comando): cite_audit + coverage + quality + recall + emite la muestra para verificación semántica. Cero `sím✗` (métodos inexistentes); arreglar todo REFUTED del paso semántico.

## Por qué este método evita alucinar

- La unidad de trabajo es **una pregunta con respuesta binaria contra fuente**, no "escribe lo que sepas del módulo". Un "no" o un "no encontrado" son salidas legítimas → no hay presión a rellenar.
- Cada afirmación lleva ancla de símbolo verificable por `cite_audit --methods`. Los errores históricos del KB (≈25 métodos inexistentes) salieron justo de escribir sin anclar; el gate los caza.
- Reorganizar/consolidar contenido YA verificado (tipo "Path A": mover bloques v19/config a su fichero) es riesgo ≈0 y suele ser la mejora de mayor ROI: el conocimiento ya existe, solo no era encontrable por el flujo del SKILL.

## Versión nueva de Odoo

Cuando sale vN+1 NO se relee el KB: se procesa el *diff* de símbolos caídos. Procedimiento en [`VERSION-BUMP.md`](VERSION-BUMP.md) (motor = `cite_audit --methods` contra la fuente nueva).

## Citas out-of-roots: NO son drift (excepciones conocidas)

`cite_audit`/`quality` resuelven contra los roots por defecto. Estos incluyen **v18+v19 CE+EE + OCA (`$OCA_SRC/v18,v19`) + indaws (`$PROJECTS_ROOT/indawsodoo/18`)** (probados si existen) — así los dominios `ecosystem/` (oca-accounting, indaws, inbuilt, l10n_es) y los core que citan OCA (p.ej. `cron`→`queue_job`) resuelven contra su fuente REAL, no contra un fichero core homónimo. Único caso que aún sale como `missing` SIN ser error:
- **`account_consolidation` (~84) y `consolidation_coa` (2)** — módulo **v17-only** (eliminado en v18). Su fuente está fuera de cualquier root v18/v19. `kb_release_check` ya excluye account_consolidation del gate de métodos.

Lo que SÍ es drift y se arregla (Fase E del refactor): línea fuera de rango con la fuente presente, o nombre de fichero abreviado/incorrecto (p.ej. `ubl_bis3.py`→`account_edi_ubl_cii/models/account_edi_xml_ubl_bis3.py`, `mrp_workorder_iot.py`→`mrp_workorder_iot/models/mrp_workorder.py`). Se re-ancla a `fichero/correcto.py → símbolo`.

## Atajo de diagnóstico (¿un dominio está flojo por cobertura o por estructura?)

`quality.py` puntúa `3.0 + 3.5·verified + 2.0·min(gotchas/50) + 1.0·min(files/6)`. Un dominio bajo puede serlo por **estructura** (faltan ficheros del patrón → extraer/consolidar, riesgo 0) o por **cobertura** (faltan gotchas/capacidades → ciclo de arriba). Mira primero `coverage.py`: si tiene <5 ficheros pero buen contenido embebido, es estructura (barato); si tiene pocos gotchas y poco texto, es cobertura (caro, requiere fuente).
