---
name: indaws-odoo-wizard-builder
version: 0.2.10
effort: medium
model: opus
description: >-
  Construye wizards Odoo (TransientModel): target=new, default_get con active_ids, botones que devuelven acciones, form con footer y ACL. Usala cuando pidas un asistente. Incluye wizards multi-paso y botones que abren otro wizard (v17-v19).
allowed-tools: [Read, Write, Edit, Bash, Grep, Glob]
---

# Odoo Wizard Builder

Guía para construir asistentes Odoo (`models.TransientModel`): apertura, precarga
de contexto, botones de acción, vistas, campos temporales, seguridad y vacuum.
Base v18; diffs v17/v19 en `references/version-diffs.md`. NO inventes API: para
firmas y parámetros consulta `odoo-version-api-reference`.

Estándares compartidos: `references/shared/coding-standards.md` (código e
identificadores SIEMPRE en inglés; strings de cara al usuario traducibles).

## Workflow

1. **Definir el propósito**: un wizard es un formulario efímero que actúa sobre
   registros existentes (reversar asientos, importar, confirmar en lote...). Si
   los datos deben persistir, NO es un wizard → usa un modelo normal.
2. **Modelo** (`references/transient-model.md`): `_name`, `_description`,
   herencia `models.TransientModel`. Campos delegados a `odoo-orm-modeler`.
3. **Apertura** (`references/context-and-defaults.md`): botón/acción que devuelve
   `ir.actions.act_window` con `target="new"`, `res_model`, `view_mode="form"`,
   `context` (`default_*`, `active_id/active_ids/active_model/active_domain`).
4. **Precarga** (`references/context-and-defaults.md`): `default_get(self,
   fields_list)` lee `self.env.context` para inicializar campos desde la
   selección de origen. Patrón real de `account.move.reversal`.
5. **Botones de acción** (`references/actions-and-multistep.md`): métodos
   `type="object"` que devuelven un dict (cerrar, recargar, encadenar wizard,
   lanzar report) o `special="cancel"` en la vista.
6. **Vista** (`references/wizard-views.md`): `<form>` con `<footer>` para los
   botones; visibilidad por `state` en multi-paso. Delegable a `odoo-view-designer`.
7. **Seguridad** (`references/transient-model.md` §ACL): un `ir.model.access`
   amplio (CRU para el grupo funcional). Delegable a `odoo-security-designer`.

## GOTCHAS

1. **Los registros transient son efímeros, pero no en una hora** — el vacuum
   (`_transient_vacuum`) NO se dispara al crear registros: solo lo llama el cron
   `base.autovacuum_job`, que corre **una vez al día**. El límite de edad de 1 h
   se aplica cuando el cron pasa, así que la fila puede vivir ~24 h (medido). Y
   por debajo de **5 minutos** no borra nada, digas lo que digas en
   `_transient_max_hours`. Aun así: NUNCA guardes ahí datos que deban durar.
2. **`active_ids` vs `active_id`** — botón sobre 1 registro pasa `active_id`;
   acción de lista multi-selección pasa `active_ids`. En `default_get` lee
   `active_ids` (lista) y comprueba `active_model` antes de hacer `browse`.
3. **Devolver `False`/`None` cierra el wizard** — si un botón no devuelve acción,
   Odoo cierra la ventana. Para mantenerla abierta devuelve una acción explícita.
4. **`ensure_one()` en los botones** — los métodos de acción operan sobre un solo
   wizard; empieza con `self.ensure_one()` salvo que proceses varios a propósito.
5. **m2o de modelo NORMAL → transient = PROHIBIDO** — y no es una convención:
   `ValueError: Many2one … from Model to TransientModel is forbidden` **al
   instalar el módulo**, con la base a medias (medido v18 y v19). Al revés
   (transient → modelo normal) SÍ es correcto, pero ojo al gotcha 9.
6. **El wizard necesita ACL como cualquier modelo, y no aísla por creador** — el
   docstring de Odoo dice que todos pueden crear y que cada uno solo ve lo suyo:
   **las dos mitades son falsas** (medido). Sin `ir.model.access` no se puede ni
   crear, y con él B lee los registros que creó A. Si hace falta aislamiento, lo
   pone una `ir.rule` con `[('create_uid', '=', user.id)]`.
7. **`default_get` recibe `fields_list`** — comprueba `if 'campo' in fields_list`
   antes de asignar `res['campo']`; respeta el `super().default_get(...)`. Trae
   **solo los campos que faltan** en el `vals` del llamante, no todos.
8. **Recargar la vista origen** tras actuar: devuelve un `act_window` al modelo
   real (`res_model`, `res_id`/`domain`) o `{'type': 'ir.actions.client',
   'tag': 'reload'}` — no asumas que la lista se refresca sola.
9. **Un m2o `required` del wizard es `ondelete='cascade'` por defecto**, no
   `restrict` como sería en un modelo normal (`odoo/fields.py:3241-3249`, `Many2one.setup_nonrelated`). Borrar el
   destino **pasa** y se lleva la fila del wizard. Un wizard no protege nada de
   lo que apunta.
10. **`default_get` corre también en un `create()` por ORM** — con el contexto
   que lleve el env. Bueno para probarlo sin navegador; malo si validas ahí con
   `UserError` y algún proceso interno crea el wizard sin selección.

## NEVER
- NUNCA persistir datos de negocio en un TransientModel esperando durabilidad.
- NUNCA crear un Many2one desde un modelo normal hacia un transient.
- NUNCA omitir el `ir.model.access` del wizard.
- NUNCA inventar firmas/parámetros de API: consulta `odoo-version-api-reference`.
- NUNCA escribir identificadores/código en otro idioma que no sea inglés.

## ALWAYS
- SIEMPRE abrir el wizard con `target="new"` (popup modal).
- SIEMPRE leer la selección de origen vía `default_get` + context, no por SQL.
- SIEMPRE `ensure_one()` en los botones de acción mono-registro.
- SIEMPRE devolver una acción explícita si el flujo debe continuar (multi-paso).
- SIEMPRE poner los botones en `<footer>` dentro del `<form>`.
- **Provenance marker (ALWAYS):** every generated file (except `.csv`/`.json` — no comment syntax) ends with `Made with odoo-wizard-builder · v<version del frontmatter de esta skill>`, wrapped in the file-type comment, idempotent. Full rule: `references/shared/skill-provenance-marker.md`.

## Delegates to
| Skill | Para qué |
|-------|----------|
| odoo-orm-modeler | Definir los campos del wizard (tipos, m2m/o2m, parámetros) |
| odoo-view-designer | Diseñar la vista `<form>` del wizard y el `<footer>` |
| odoo-security-designer | Generar el `ir.model.access` del wizard |
| odoo-version-api-reference | Firmas/parámetros de API y diffs entre versiones |
| odoo-deep-knowledge | Gotchas de TransientModel (vacuum, default_get, registros huérfanos) |

## Consumed by
| Skill | Para qué |
|-------|----------|
| odoo-module-creator | Añadir wizards al andamiaje de un módulo nuevo |
| odoo-solution-architect | Cuando el plano de solución incluye un wizard como componente |

## REFERENCES
| Fichero | Contenido | Leer cuando |
|---------|-----------|-------------|
| references/transient-model.md | TransientModel, vacuum (`_transient_max_count`/`_transient_max_hours`), ACL | Crear el modelo / dudas de persistencia o seguridad |
| references/context-and-defaults.md | Apertura `act_window`, context, `active_*`, `default_get` | Abrir el wizard y precargar campos |
| references/actions-and-multistep.md | Dicts de retorno, cerrar/recargar/report, multi-paso (`state` / encadenar) | Implementar botones y flujos multi-paso |
| references/wizard-views.md | Vista `<form>` + `<footer>`, `invisible` por `state` | Escribir la vista del wizard |
| references/version-diffs.md | Diffs v17 → v18 → v19 (ORM movido a `odoo/orm/`) | Trabajar en una versión != v18 |
