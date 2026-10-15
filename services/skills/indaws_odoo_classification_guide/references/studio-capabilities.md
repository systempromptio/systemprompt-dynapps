# Studio Capabilities por Version

## Cuando aparece cada mecanismo (verificado contra fuente)

Verificado 2026-08-07 contra `/opt/odoo/odoo{14..19}` (CE) y `/opt/odoo_enterprise/{14..19}`
(EE, `web_studio`). **Existe desde** = la primera version del checkout donde el mecanismo esta en
el codigo, con su cita.

| Mecanismo | Existe desde | Donde |
|-----------|--------------|-------|
| Campos custom (`ir.model.fields` con `x_`) | 14.0 | `base/models/ir_model.py` |
| Editor de vistas de Studio | 14.0 | `web_studio/static/src/` (client action) |
| **Python en automatizaciones** | **14.0** | `base.automation.action_server_id` es M2o a `ir.actions.server` con `delegate=True` (`base_automation/models/base_automation.py:38-41`, `DATE_RANGE_FACTOR` en v18; en v14 era el campo `action_server_id`) → hereda `state='code'` (`base/models/ir_actions.py:409`, `class IrActionsActWindowView` v14, "Execute Python Code") |
| **Campos computados por formula** | **14.0** | `ir.model.fields.compute` = `fields.Text("Code to compute the value of the field")` + `depends` (`odoo/addons/base/models/ir_model.py:513`, `IrModel._add_manual_models`, y `:518`, mismo método — v14 citaba ahí el campo `compute`), evaluado con `safe_eval` (`:35`) |
| **Approval flows de Studio** | **14.0** | `studio.approval.rule` (`web_studio/models/studio_approval.py:16`, `StudioApprovers` v14): `group_id`, `domain` condicional, `exclusive_user`, `message`, entries |
| Aprobadores nominales + delegacion | 18.0 | v18 añade `approver_ids`, `approval_group_id`, `is_delegation`, `approver_log_ids` — los cuatro con **0 ocurrencias en v14-v17**. El fichero pasa de 804 lineas (v17) a 1308 (v18) y de **30 a 40** campos, contando `grep -cE '^\s*\w+ = fields\.' models/studio_approval.py` |
| **Dashboards** | **16.0** | `spreadsheet_dashboard` (CE) + `spreadsheet_dashboard_edition` (EE); ninguno de los dos existe en 14.0/15.0 |
| **Crear apps desde Studio** | **14.0** | `web_studio/static/src/js/app_creator.js` v14 (movido a `client_action/app_creator/` en v15+) |
| Editor de informes de Studio | 14.0 | `web_studio/static/src/js/reports` + `xml/report_editor.xml` v14; reescrito como `client_action/report_editor` en 17.0 |

**⚠️ Lo que esta tabla NO dice.** Que el mecanismo exista en el codigo **no** prueba que la UI de
Studio lo exponga en esa version, ni que sea buena idea usarlo. Concretamente: **no se pudo
verificar por lectura de fuente en que version la UI de Studio ofrece la caja de formula de un campo
computado** — `web_studio` no menciona `compute` como propiedad de campo en NINGUNA version, ni en
17.0 ni en 19.0 (el fichero `type_specific_and_computed_properties.js` de v18/v19 va de propiedades
del sidebar que dependen de otras, no de compute fields). Esa gradacion solo se resuelve abriendo
Studio en cada version. Si el caso depende de ello, **probarlo en la instancia**, no citar esta
tabla.

**Gradaciones de calidad ("basico" / "mejorado" / "muy limitado"): son juicio, no dato.** Esta
referencia tenia una matriz 8×6 de esas etiquetas; se retiro porque cuatro de sus filas situaban la
capacidad **hasta cuatro majors mas tarde** de lo que muestra la fuente (Python en automatizaciones
en 18.0, computados en 17.0, approvals en 18.0, apps en 17.0 — todos existen en 14.0). Para
clasificar, usa la tabla de arriba mas los umbrales de la seccion siguiente.

**Nota de doctrina (independiente de la version):** que Studio *pueda* hacer algo no lo convierte en
la via correcta. Ver los umbrales de abajo y `ai-impact-studio-vs-custom.md`.

## Umbrales Studio vs Desarrollo

> **Doctrina (Matiz IA):** Studio es la via para lo **trivial y de bajo riesgo**. Toda **logica no
> trivial** —validaciones, flujos, integridad de datos, integraciones— va a **modulo gobernado**
> (revisado, testeado, Core-compliant), aunque "cupiese" en Studio: con IA el coste del codigo se
> desploma y un modulo es consistente y migrable, mientras Studio no tiene gate de revision. Estos
> umbrales definen esa frontera de "trivial". Ver `ai-impact-studio-vs-custom.md`.

La funcionalidad se puede resolver con Studio si cumple **TODOS** estos criterios:

| Criterio | Umbral Studio |
|----------|--------------|
| Logica Python | < 50 lineas (solo en automatizaciones; disponible desde 14.0, no desde 18.0) |
| Modelos nuevos | 0 (solo extiende modelos existentes) |
| Logica de negocio | Simple: condiciones, asignaciones, envio de emails |
| Vistas | Modificacion de existentes, no completamente nuevas |
| Seguridad | Grupos estandar Odoo, sin record rules complejas |
| Frontend JS/OWL | No requiere |
| APIs externas | No requiere |
| Reportes PDF custom | No requiere (o modificaciones menores) |
| Wizards | No requiere |

### Señales de alerta: Reclasificar de Studio a Custom si...

- La automatizacion Python en Studio supera 30 lineas
- Se necesita herencia entre modelos
- Se necesita override de metodos existentes
- Hay logica de negocio condicional compleja (>3 niveles de anidacion)
- Se necesitan tests automatizados
- La configuracion Studio es muy compleja (>15 campos + multiples automatizaciones)

## Ventajas y Desventajas de Studio

**Ventajas:**
- Implementacion rapida (horas vs dias)
- No requiere desarrollador
- Interfaz visual comprensible para el cliente
- Incluido en licencia Enterprise

**Desventajas:**
- Migrabilidad: cambios Studio pueden romperse entre versiones
- Limitado en logica compleja
- Sin control de versiones (no hay git)
- Dificil de debuggear
- No soporta tests automatizados
- No soporta herencia entre modelos
