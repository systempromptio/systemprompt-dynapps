# Skill Granularity — ¿una skill grande, mini-skills, u orquestador?

Marco de decisión para el tamaño/partición de skills. Alineado con la guía oficial
de Anthropic (*Skill authoring best practices*).

## Principio
**Una skill = una CAPACIDAD con un disparador reconocible.** El eje NO es
"grande vs pequeña", es **capability (trigger) vs volumen (references)**.

- El **volumen** de contenido se resuelve con **progressive disclosure** (references
  por dominio, 1 nivel, `SKILL.md` <150 líneas / <500 oficial), NO partiendo en más skills.
  Las references no cuestan contexto hasta que se leen.
- Solo la `description` (≤1024 chars) se carga **SIEMPRE** y es el **selector** entre
  100+ skills. Cada skill extra = una descripción permanente en contexto + más riesgo
  de colisión de triggers. Por eso micro-skills no es gratis.

## Cuándo PARTIR en otra skill (vs poner una reference)
Parte solo si se cumple **(a) o (b)**:
- **(a) Trigger distinto**: se invoca con una intención/"when to use" claramente
  diferente (otra clase de tarea).
- **(b) Reuso real**: la usan ≥2 consumidores como capacidad compartida (SSOT) —
  p.ej. `odoo-version-api-reference`, `odoo-deep-knowledge`.

Si es solo "mucho contenido del mismo flujo" → **NO partas**: usa references.
Si dos skills **siempre** se usan juntas y nunca sueltas → **júntalas**.

## Monolito vs micro (tradeoffs)
| | Skill grande (con references) | Muchas mini-skills |
|---|---|---|
| Discovery | descripción tiende a vaga → peor selección | precisa, pero N descripciones siempre cargadas |
| Contexto | ligero (references on-demand) | coste fijo por cada descripción |
| Mantenim. | un sitio | superficie ×N, posible drift |
| Reuso | no reutilizable por piezas | piezas reutilizables |
| Coste runtime | bajo (sin saltos) | + saltos si se orquesta |

## Cuándo introducir un ORQUESTADOR (skill/agente que delega)
Un orquestador-que-delega-en-subagentes **NO es un primitivo de la doc de skills**;
es un patrón de Claude Code agents que **añade coste** (saltos de subagente,
latencia, hand-off de contexto). Justifícalo solo si:
- Las sub-capacidades son **independientemente disparables/reutilizables** (cada una
  vale como skill por (a)/(b)), **y/o**
- Hay **paralelismo** o fan-out real (varias ramas a la vez), **y/o**
- El flujo es largo y conviene **aislar contexto** por fase.

Si el flujo es **lineal y de una sola intención** → una skill con references es más
barata y simple que orquestar. No orquestes por defecto.

## Checklist de decisión (al diseñar/añadir)
1. ¿Intención/trigger distinto de lo existente? Si no → reference en la skill que ya existe.
2. ¿La usarán ≥2 skills? Si sí → skill aparte (SSOT). Si no → reference.
3. ¿Solo es volumen? → references, no nueva skill.
4. ¿Siempre van juntas? → fusiónalas.
5. ¿Necesitas reuso independiente o paralelismo? → orquestador. Si no → single skill.
6. Mide con **uso real** antes de consolidar/partir: sin datos de uso, prefiere
   menos skills (menos descripciones siempre cargadas).

## Anti-patrones
- Partir por **volumen** (debería ser reference).
- Crear orquestador para un flujo **lineal** de una sola intención.
- Mega-skill cuya `description` intenta cubrir "todo" → mala discovery.
- Sobre-split: familia de mini-skills que **nunca** se usan por separado.
