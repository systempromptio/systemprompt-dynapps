# odoo-com-sync — Workflow Detail

## Conexión — conector MCP de Odoo

El CRM de Indaws/Dynapps se lee/escribe **exclusivamente** por el **conector MCP de Odoo** (tools `whoami`,
`orm_search_read`, `orm_write`, `orm_create`). **NO** usar curl, `odoo-pilot`, `search_records.sh` ni
JSON-RPC/JSON2. El backend ya no está en `https://www.indaws.es` (302 → dynapps.es) ni en
`https://www.portal.dynapps.es` (web pública Next.js). Verificar la sesión con `whoami` antes de empezar; si el
conector no está disponible → reportar y abortar. Datos de la instancia: DB `master`, usuario `victor.peris@dynapps.es`.

## Configuración de etapas y scope (SIEMPRE por ID)

**REGLA DE ORO: razona y filtra etapas por `id`, NUNCA por nombre.** Las etapas del CRM se renombran y reordenan (la `sequence` también cambia). El `id` es inmutable. Toda decisión (qué sincronizar, detectar cambio de etapa, mostrar la etapa) se hace sobre `stage_id[0]` (el entero) y el nombre se resuelve desde esta tabla canónica.

### Mapa canónico de etapas (`crm.stage`)

| id | Etapa | rol en el pipeline |
|----|-------|--------------------|
| 1  | Lead | entrada / sin cualificar |
| 26 | Cualificado | cualificada |
| 3  | Propuesta | propuesta enviada |
| 27 | Negociación | negociación |
| 29 | En análisis | análisis técnico |
| 28 | En Espera | parada / on-hold |
| 4  | Ganado | **ganada** (`is_won = true`) |

> Si una query devuelve un `stage_id` cuyo id no está en esta tabla, el CRM ha cambiado: vuelve a leer `crm.stage` (ver `api-examples.md` → "Obtener etapas") y actualiza el mapa **antes** de seguir.
>
> El mapa antiguo de Indaws `[22, 19, 12, 24, 13, 8, 2, 15, 9, 3, 7, 23]` está **OBSOLETO** desde la migración a Dynapps; no usarlo. El scope verificado en vivo el 30/06/2026 es `stage_id in [26, 3, 27, 29]`.

### Scope de sincronización al portal — `stage_id in [26, 3, 27, 29]`

Solo se sincronizan oportunidades del **pipeline cualificado**: Cualificado (26), Propuesta (3), Negociación (27), En análisis (29). Se **excluyen** por ID:
- `1` Lead → aún sin cualificar, no aporta valor al Partner Manager de Odoo.
- `28` En Espera → parada; no es actividad viva.
- `4` Ganado → ya cerrada (sustituye al antiguo filtro frágil `probability < 100`).

Este filtro de etapa por ID es el criterio único de "no ganada / activa"; ya no se usa `probability < 100`.

---

## Fase 1 — Nuevas oportunidades (última semana)

### Paso 1.1 — Extraer oportunidades de Indaws (vía MCP)

Verifica primero la sesión con `whoami`. Calcula `create_date >=` el lunes de la semana actual y usa el tool
`orm_search_read` (ver `api-examples.md` para el payload completo). Filtros críticos:
- `type = opportunity` — excluye leads sin cualificar
- `active = true` — excluye archivadas/perdidas
- `stage_id in [26,3,27,29]` — **scope por ID** (pipeline cualificado): excluye Lead (1), En Espera (28) y Ganado (4). Sustituye al antiguo `probability < 100`. Ver "Configuración de etapas y scope".
- `email_from != false` — solo oportunidades con email (criterio de cribado)

Campos: `id, name, partner_name, email_from, phone, stage_id, date_deadline, probability, create_date, user_id, description, expected_revenue, contact_name, activity_ids, x_studio_ai_toma_de_datos, x_studio_fecha_demo, x_studio_fecha_toma_datos, x_studio_toma_datos_transcript, x_studio_ai_guion_demo, x_studio_demo_users_ids, x_studio_commercial_email, x_studio_commercial_name, x_studio_commercial_phone`

### Paso 1.1b — Extraer actividades y fechas (para la descripción)

Para cada oportunidad, recopila la información de actividad que se incluirá **dentro de la descripción** al crearla (Paso 1.2 → bloque de actividad). El portal de odoo.com **ya no tiene composer de comentarios** (ver Paso 1.6), por lo que esta información va siempre en la descripción.

**a) Actividades PENDIENTES** (`mail.activity`) — la "próxima acción". Ver `api-examples.md` → "Actividades pendientes de una oportunidad". Usa SOLO `activity_type_id`, `summary`, `date_deadline`, `user_id`, `state`.

**b) Actividades CERRADAS** (histórico) — viven en el chatter como mensajes con subtipo "Actividades". Léelas con `orm_search_read` sobre `mail.message` (ver `api-examples.md` → "Extraer mensajes del chatter") y quédate con los mensajes cuyo `subtype` sea de actividad (o, de forma sencilla, las últimas 3-4 entradas relevantes con su `date`).

**Mapa de tipos de actividad por ID** (igual que las etapas: razona por `id`, no por nombre):

| id | Tipo |
|----|------|
| 59 | Toma de datos |
| 38 | Demo |
| 61 | Presentación de análisis |
| 2  | Llamada |
| 3  | Reunión |
| 4  | Por hacer |
| 1  | Correo electrónico |

**Fechas-hito a recoger** (de los campos ya traídos en 1.1): `x_studio_fecha_toma_datos`, `x_studio_fecha_demo`, y **`date_deadline` = cierre estimado**.

**HIGIENE (crítico):** **NUNCA** vuelques el campo `note` de `mail.activity` ni HTML crudo: suele contener blobs enormes (scripts de venta, HTML pegado de LinkedIn) que ensucian el portal. Texto plano resumido, sin HTML.

### Paso 1.2 — Construir descripción enriquecida (~300 palabras)

Estructura (el bloque de actividad va SIEMPRE dentro de esta descripción — no como comentario aparte, ver Paso 1.6):
```
**[Nombre empresa / Contacto]** — Oportunidad creada el [fecha]

**Resumen:** [descripción original]
**Toma de datos:** [x_studio_ai_toma_de_datos]
**Datos clave:** Responsable, Etapa (resolver stage_id[0] contra el mapa canónico — NUNCA el nombre crudo), Probabilidad, Ingresos, Fecha cierre, Fecha demo
**Ideas de apoyo:** 2-3 ideas de conversión basadas en datos, sector, etapa
**Comercial Indaws:** Nombre, Email, Teléfono

📅 ACTIVIDAD Y PRÓXIMOS PASOS
▶ PRÓXIMO: {tipo} · {fecha} · {resumen} — {responsable}
   [si la próxima actividad está VENCIDA (state=overdue) → añadir "⚠️ VENCIDA"]
   [si NO hay actividad pendiente → "⚠️ Sin actividad planificada"]
Hitos:
· Toma de datos: {x_studio_fecha_toma_datos | —}
· Demo: {x_studio_fecha_demo | —}
· Cierre estimado: {date_deadline | —}
Histórico reciente:
· {fecha} — {tipo}: {resumen}
· {fecha} — {tipo}: {resumen}
```

Resuelve `{tipo}` con el mapa de tipos por ID (Paso 1.1b), usa fechas en `DD/MM/AAAA`, texto plano sin HTML, y omite las líneas de histórico si no hay datos. Nota: `date_deadline` no existe como campo en el portal → por eso va en el texto de la descripción.

### Paso 1.3 — Navegar al portal odoo.com

Navigate a `/es_ES/my/opportunities`. Verificar sesión: si aparece "Cerrar sesión" o el nombre del usuario, la sesión está activa — NO reintentar login.

### Paso 1.4 — Crear cada oportunidad (proceso de dos pasos)

**Paso A — Modal de creación (solo 3 campos):** `contact_name`, `title`, `description` (con el bloque de actividad ya dentro); botón `.new_opp_confirm`. Usar `waitForElement()` con MutationObserver para esperar el DOM — no `setTimeout` fijo.

**Paso B — En la página de detalle, completar email/teléfono** usando `.edit_contact_confirm` (formulario de contacto: `partner_name`, `phone`, `email_from`, `street`, `street2`, `city`, `state_id`, `zip`, `country_id`). **Este formulario recarga la página al enviar.** Ver `js-patterns.md` para `setVal()`, `crearOportunidad()` y `actualizarContacto()`.

Para verificar que un valor se guardó: `document.querySelector('[name="email_from"]').value` (los inputs no aparecen en `get_page_text`).

### Paso 1.5 — Verificar creación

URL debe contener `/my/opportunity/`. Registrar slug/ID resultante.

### Paso 1.6 — (Sin comentario) — la actividad ya está en la descripción

El portal de partner de odoo.com **NO tiene composer de comentarios** (`#chatterRoot` queda vacío, no monta) **ni campo `description` editable en el detalle** (verificado en producción 30/06/2026). Por tanto **no hay paso de "publicar comentario"**: toda la información de actividad / próximos pasos / hitos / cierre estimado ya quedó incluida **dentro de la descripción** que se setea al crear la oportunidad (Paso 1.2). No intentar publicar un comentario aparte ni esperar a que monte ningún chatter.

---

## Fase 2 — Actualizar oportunidades (últimos 30 días)

Dado que el portal no tiene composer de comentarios ni campo `description` editable en el detalle, en Fase 2 **solo** se puede refrescar `expected_revenue` y `probability` (formulario `.edit_opp_confirm`). No se puede reescribir la descripción ni publicar historial desde el detalle.

### Paso 2.1 — Extraer oportunidades activas de los últimos 30 días (vía MCP)

`create_date >=` 30 días atrás. Mismos filtros que Fase 1 (`stage_id in [26,3,27,29]`, scope por ID). Bastan los campos `id, name, partner_name, email_from, stage_id, expected_revenue, probability`. Ver `api-examples.md`.

### Paso 2.2 — Buscar en odoo.com por email

Listar en `/es_ES/my/opportunities`, buscar por email. Si existe: actualizar (Paso 2.3). Si no: crear (Fase 1).

### Paso 2.3 — Refrescar ingresos y probabilidad (`.edit_opp_confirm`)

Formulario `.edit_opp_confirm`: setear `expected_revenue` y `probability` (NO contiene `description`). Ver `js-patterns.md` → `actualizarOportunidad()`.

---

## Patrón Orchestrator + Executor

Para sesiones con muchas oportunidades (>10 o con errores):
- **Orchestrator (Opus, thinking high):** extrae oportunidades vía MCP, genera descripciones en batch (con el bloque de actividad dentro), produce JSON con plan `[{accion, datos}]`, recoge resultados.
- **Executor (Sonnet, thinking medium):** recibe un item, navega odoo.com, ejecuta acción (crear / refrescar ingresos+probabilidad), verifica, retorna éxito/error.

Para sync rutinario (<10 oportunidades): Sonnet con thinking medium es suficiente para todo.

## Flujo de ejecución completo

```
1. Leer credenciales odoo.com (.env). Verificar el conector MCP de Odoo con whoami.
2. Confirmar: ¿Fase 1, Fase 2, o ambas?
3. Extraer oportunidades de Indaws vía MCP (orm_search_read, scope por ID: stage_id in [26,3,27,29])
4. Fase 1: por cada oportunidad → extraer actividades/fechas (1.1b) → generar descripción CON el bloque
   de actividad dentro (etapa por ID) → crear en odoo.com (modal→detalle: email/teléfono) → verificar
   (NO hay comentario que publicar — Paso 1.6)
5. Fase 2: buscar por email → refrescar expected_revenue y probability (.edit_opp_confirm, 2.3)
   → o crear si no existe (NO se reescribe descripción ni se publica historial)
6. Confirmar: N creadas, M actualizadas (ingresos/probabilidad)
```
