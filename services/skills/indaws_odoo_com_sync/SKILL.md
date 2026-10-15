---
name: indaws-odoo-com-sync
version: 1.2.3
effort: high
model: opus
description: >-
  Sincroniza oportunidades del CRM Indaws/Dynapps (Odoo 19 EE) con el portal partner de odoo.com en
  dos fases: Fase 1 crea las nuevas de la última semana; Fase 2 actualiza las de las últimas 4
  semanas (ingresos esperados y probabilidad). El scope se decide por etapa (id), nunca por nombre
  ni probabilidad; el CRM se lee/escribe vía el conector MCP de Odoo, no curl. Triggers:
  'sincronizar oportunidades', 'sync CRM odoo.com', 'pasar oportunidades a odoo.com', 'actualizar
  portal de partner', 'sync odoo.com'.
---

# odoo-com-sync

Sincroniza oportunidades de Indaws/Dynapps CRM (Odoo 19 Enterprise) con el portal de partner de odoo.com. Ver `references/workflow-detail.md` para el detalle completo de pasos, mapa de etapas, actividad en la descripción y llamadas MCP.

## Agent Routing

| Rol | Modelo | Uso |
|-----|--------|-----|
| **Orchestrator** | Opus (thinking high) | Extraer datos Indaws, generar descripciones, decidir qué sync, manejar errores, >10 oportunidades |
| **Executor** | Sonnet (thinking medium) | Navegar odoo.com, rellenar formularios (modal de creación, contacto, ingresos/probabilidad) (tareas mecánicas) |

Para sync rutinario (<10 oportunidades): Sonnet con thinking medium para todo.

## Scope de sincronización (por ID de etapa)

Solo se sincroniza el pipeline cualificado: `stage_id in [26, 3, 27, 29]` (Cualificado, Propuesta, Negociación, En análisis). Se excluyen Lead (1), En Espera (28) y Ganado (4). Mapa canónico en `references/workflow-detail.md`.

## Workflow

1. **Credentials**: CRM de Indaws/Dynapps vía **conector MCP de Odoo** (`whoami`/`orm_search_read`/`orm_write`/`orm_create`; verificar con `whoami` antes de empezar); odoo.com de `.env` del directorio de trabajo.
2. **Confirmar fase**: ¿Fase 1 (últimos 7 días), Fase 2 (últimos 30 días), o ambas?
3. **Extraer Indaws** (MCP `orm_search_read`, ver `references/api-examples.md`): `type=opportunity, active=true, stage_id in [26,3,27,29], email_from!=false`, con campos `x_studio_*`.
4. **Extraer actividades/fechas** (Paso 1.1b): `mail.activity` pendientes + actividades cerradas del chatter + fechas clave (toma de datos, demo, cierre).
5. **Fase 1**: generar descripción ~300 palabras CON el bloque de actividad dentro → crear en odoo.com (modal 3 campos: contact_name/title/description → detalle para email/teléfono). NO hay comentario que publicar. Ver `references/workflow-detail.md`.
6. **Fase 2**: buscar en odoo.com por email → refrescar `expected_revenue` y `probability` (`.edit_opp_confirm`). No se puede reescribir la descripción ni publicar historial. Ver `references/workflow-detail.md`.
7. **Confirmar resultado**: N creadas, M actualizadas (ingresos/probabilidad).

## GOTCHAS

1. **Filtrar etapas por `id`, NUNCA por nombre** — las etapas (`crm.stage`) y tipos de actividad se renombran/reordenan; el `id` es inmutable. El scope es `stage_id in [26,3,27,29]`. Si aparece un id de etapa desconocido → el CRM cambió: re-leer `crm.stage` y actualizar el mapa antes de seguir. Nunca filtrar por `stage_id.name` ni por `probability < 100` (frágil: rompió la sync al renombrar etapas).
2. **CRM vía conector MCP de Odoo, no curl** — el CRM de Indaws/Dynapps se lee/escribe SOLO con el conector MCP (`whoami`/`orm_search_read`/`orm_write`/`orm_create`). NO usar curl, `odoo-pilot`, `search_records.sh` ni JSON-RPC/JSON2. El backend ya no está en `https://www.indaws.es` (302 → dynapps.es) ni en `https://www.portal.dynapps.es` (web pública Next.js). Verificar con `whoami` antes de empezar; si el conector no está → reportar y abortar.
3. **Campo `expected_revenue`, no `planned_revenue`** — en Odoo 19, `planned_revenue` da error de campo inválido.
4. **Dos formularios separados en la página de detalle**: `.edit_opp_confirm` (`expected_revenue`, `probability`, prioridad — **NO `description`**) y `.edit_contact_confirm` (`partner_name`, `phone`, `email_from`, `street`, `city`, `state_id`, `zip`, `country_id`; **recarga la página al enviar**). Usar el formulario incorrecto → datos ignorados silenciosamente.
5. **Modal de creación tiene solo 3 campos** (`contact_name`, `title`, `description`; botón `.new_opp_confirm`). Email/teléfono solo se pueden añadir en la página de detalle, después de crear.
6. **El portal NO tiene composer de comentarios ni `description` editable en el detalle** (verificado 30/06/2026: `#chatterRoot` no monta). La actividad/próximos pasos/hitos/cierre van SIEMPRE **dentro de la `description`** que se setea al CREAR (modal). No intentar publicar un comentario aparte. En Fase 2 solo se refrescan `expected_revenue` y `probability`. Ver `references/workflow-detail.md`.
7. **NUNCA volcar el `note` de `mail.activity` ni HTML crudo** — contienen blobs enormes (scripts de venta, HTML de LinkedIn). El bloque de actividad de la descripción lleva solo: tipo (por id) + fecha + `summary` + responsable + estado. Resumir, no copiar literal.
8. **`setVal()` con eventos nativos, no `form_input`** — los campos Odoo usan Owl reactivo. `el.value = x` directo no persiste. Ver `references/js-patterns.md`.
9. **`waitForElement()` en vez de `setTimeout` fijo** — el DOM tarda ~600-700ms en renderizarse tras click. Usar MutationObserver. Ver `references/js-patterns.md`.
10. **`navigate` y JS deben ser llamadas separadas** — no combinar en el mismo `browser_batch`.
11. **Verificar sesión antes de login** — si la página muestra "Cerrar sesión", la sesión está activa; no reintentar login.
12. **`date_deadline` no existe en el portal** — incluirlo en el texto de la descripción.
13. **Los valores de `<input>` no aparecen en `get_page_text`** — verificar con JS: `document.querySelector('[name="email_from"]').value`.

## NEVER / ALWAYS

**NEVER:** Filtrar etapas por nombre o por `probability < 100`. Usar curl/`search_records.sh`/JSON2/odoo-pilot para el CRM (solo conector MCP). Mezclar los dos formularios de detalle. Intentar publicar un comentario en el portal o editar la descripción en el detalle (no existe — G6). Usar `setTimeout` fijo para esperar el DOM. Volcar el `note` de actividades o HTML crudo en la descripción.
**ALWAYS:** Filtrar el scope por `stage_id in [26,3,27,29]` (por id). Leer/escribir el CRM vía conector MCP de Odoo (verificar `whoami` antes). Verificar sesión antes de navegar a login. Incluir `"context": {"lang": "es_ES"}` en todas las llamadas. Meter el bloque de actividad dentro de la descripción al crear. Confirmar creación verificando que la URL contiene `/my/opportunity/`.

## Delegates to

None — opera directamente con el conector MCP de Odoo (CRM Indaws/Dynapps) y el portal odoo.com (browser automation).

## Consumed by

| Skill | Suite | For what |
|-------|-------|----------|
| tl-assistant | common | Sync semanal de oportunidades CRM |

See `references/shared/odoo-connection.md` for Odoo connection patterns (conexión vía conector MCP de Odoo).

## References

- `references/workflow-detail.md` — Workflow completo Fase 1/2, mapa canónico de etapas, extracción de actividades (1.1b), actividad dentro de la descripción, patrón orchestrator+executor
- `references/js-patterns.md` — `setVal()`, `waitForElement()`, `crearOportunidad()`, `actualizarContacto()`, `actualizarOportunidad()` (ingresos/probabilidad), snippets testeados
- `references/api-examples.md` — Payloads `orm_search_read` (conector MCP) para cada modelo (crm.lead con scope por etapa, mail.activity, mail.message, crm.stage)
