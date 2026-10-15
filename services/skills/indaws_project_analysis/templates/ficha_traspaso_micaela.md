# Ficha de traspaso a Micaela — {NOMBRE_CLIENTE}

> Plantilla de la ficha de traspaso operativo. NO es el análisis funcional: es un resumen
> comercial/administrativo para que Micaela prepare el traspaso a Crol.
> Ver ejemplo completo (golden, congelado) en `../references/golden-example-traspaso-micaela.md`.

Cada campo lleva su origen entre `[]`, en orden de prioridad — prueba el primero, si no
aparece cae al siguiente:

- `[ANÁLISIS]` → ya se sabe por el análisis funcional recién generado (Paso 3A/2B). Rellénalo tú, no lo preguntes.
- `[OPORTUNIDAD]` → búscalo PRIMERO en la oportunidad de Odoo (`crm.lead` + `res.partner` del cliente) antes de preguntar nada — ver `../references/workflow.md` § Paso 6, sub-paso 2. Si el campo no existe en esta instancia o la oportunidad no tiene el dato, cae a `[PREGUNTAR]`.
- `[PREGUNTAR]` → dato comercial/relacional que ni el análisis ni la oportunidad contienen. Pregúntalo SIEMPRE en bloque (una sola tanda, no una a una), y solo lo que siga sin resolver tras los dos pasos anteriores.

---

## Datos de cliente

- Cliente (razón social, marca comercial, CIF): `[OPORTUNIDAD]`
- Fecha de alta en base de datos / apertura de oportunidad: `[OPORTUNIDAD]`
- Nombre de contacto (SPOC) y cargo: `[OPORTUNIDAD]`
- Correo de contacto (cliente): `[OPORTUNIDAD]`
- Teléfono — intentar que sea el número directo del responsable del proyecto: `[OPORTUNIDAD]`
- Sector / Industry: `[OPORTUNIDAD]`
- Ubicación: `[OPORTUNIDAD]`
- Tamaño de empresa (pequeña, mediana, gran empresa; facturación si se conoce): `[OPORTUNIDAD]`
- Número total de empleados: `[OPORTUNIDAD]`

## Usuarios y herramientas actuales

- Usuarios previstos en Odoo: `[ANÁLISIS]` (si el análisis lo recoge) o `[PREGUNTAR]`
- Herramientas que utilizan actualmente (cuál y para qué): `[OPORTUNIDAD]` — revisa descripción/notas de la oportunidad; el análisis técnico solo ve las que quedan sustituidas o integradas, no el panorama completo
- ¿Cuenta con alguna subvención?: `[OPORTUNIDAD]`
- ¿Existe intención de integrar toda la operativa en Odoo?: `[ANÁLISIS]`

## 1. Información administrativa y técnica

- Descripción general del proyecto: `[ANÁLISIS]`
- Consultoría previa realizada / horas invertidas: `[OPORTUNIDAD]`
- Duración estimada de la implementación: `[ANÁLISIS]` (si `project-plan` ya corrió) o `[PREGUNTAR]`
- Customizaciones / desarrollos previstos (a alto nivel, SIN el detalle de sección 2 del análisis): `[ANÁLISIS]`

## 2. Fases de implementación

- ¿Qué módulos se van a implementar?: `[ANÁLISIS]`
- ¿Cuándo está prevista la implementación de cada módulo?: `[ANÁLISIS]` si ya hay secuencia definida, si no `[PREGUNTAR]`
- ¿En qué módulos o fases serán necesarios desarrollos/customizaciones?: `[ANÁLISIS]`
- Fechas estimadas de cada fase: `[PREGUNTAR]` si no están cerradas en el análisis

## 3. Notas adicionales

- Previsión de crecimiento del cliente: `[OPORTUNIDAD]`
- Reticencia o predisposición a la multianualidad: `[OPORTUNIDAD]`
- Posibilidades de upselling / cross-selling: `[OPORTUNIDAD]` (y puedes sugerir candidatos si el análisis dejó GAPs aplazados o fuera de alcance; confirma siempre con el consultor)
- Presupuesto: ¿ajustado, flexible, pendiente de definir?: `[OPORTUNIDAD]`
- ¿Cliente conflictivo o con alguna consideración especial?: `[OPORTUNIDAD]`
- Cualquier otra información relevante sobre el cliente o el proyecto: `[OPORTUNIDAD]`

---

## Salida esperada

1. La ficha rellena (bloque anterior, sin los marcadores de origen).
2. Un borrador de correo a Micaela con el mismo formato que el golden example: tabla
   "Datos de cliente" + secciones narrativas por bloque. Tono profesional-cercano, cierre
   tipo "Cualquier cosa que necesites ampliar, me dices."
3. Presentar el borrador al consultor para revisión. **NUNCA enviarlo sin confirmación
   explícita** — ver GOTCHA correspondiente en `../SKILL.md`.
