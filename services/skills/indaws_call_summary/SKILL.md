---
name: indaws-call-summary
description: >
  Procesa notas o transcripciones de llamadas comerciales y genera resumen
  ejecutivo, acciones con responsables, email de seguimiento y actualizaciones
  para el CRM de Odoo. Opcionalmente publica el resumen completo como HTML con
  marca Indaws en el chatter de la oportunidad y actualiza etapa, probabilidad y
  actividades en Odoo.
  Usar cuando el usuario pida: resumir llamada, call summary, procesar notas
  de llamada, que paso en la reunion, siguiente paso tras llamada.
version: 1.2.2
model: sonnet
effort: low
---

# Call Summary

Procesa notas de llamadas comerciales en outputs estructurados: resumen, acciones, email de seguimiento y actualizacion CRM.

## Workflow

### Paso 0: Recibir Notas
PREGUNTAR al usuario:
1. **Notas o transcripcion de la llamada** (pegar directamente o indicar archivo)
2. **Empresa y oportunidad** (nombre o ID en Odoo)
3. **Tipo de llamada**: discovery, demo, propuesta, seguimiento, negociacion
4. **Asistentes** (quienes estaban en la llamada)

> **DETENTE** y espera que el usuario proporcione las notas.

### Paso 1: Identificar Oportunidad en Odoo
See references/shared/odoo-connection.md for connection pattern.
Usando odoo-pilot, buscar crm.lead por partner_name, name o ID. Campos: id, name, stage_id, expected_revenue, probability, user_id, partner_id, partner_name, date_deadline. Si no se encuentra, preguntar si crear nueva.

### Paso 2: Procesar Notas
Analizar y extraer: temas tratados, decisiones, objeciones, info nueva sobre la cuenta, proximos pasos, senales de compra/riesgo.

### Paso 3: Generar Outputs
Construir el **RESUMEN COMPLETO** (bloque entero desde `===` hasta `===`) usando el template de `references/output_templates.md`. Contiene las 5 secciones:
1. Resumen ejecutivo (3-5 puntos clave)
2. Tabla de acciones (responsable + fecha)
3. Email de seguimiento (texto plano, breve: se lee de un vistazo)
4. Nota interna
5. Actualizacion CRM sugerida (etapa, probabilidad, actividad)

Presentar este bloque de texto al usuario para revision. Al publicar en Odoo (Paso 4.1) se renderiza como **HTML con marca Indaws** (ver template HTML en `references/output_templates.md`).

### Paso 4: Aplicar en Odoo (tras aprobacion)
> **DETENTE** y espera aprobacion del usuario para cada accion.

Si aprueba, usando odoo-pilot:
- **4.1** Publicar el **RESUMEN COMPLETO** como nota HTML en el chatter de la oportunidad, usando la plantilla HTML con marca Indaws de `references/output_templates.md` (estilos inline, componentes `iw-*`, colores `#1C265D`/`#E5B92B`). Usar `crm.lead.message_post` (o `mail.message` con `message_type=comment` y `subtype_xmlid="mail.mt_note"`). Rellenar los placeholders, escapar entidades del contenido (`&`->`&amp;`, `<`->`&lt;`, `>`->`&gt;`), usar `<br>` en el cuerpo del email y omitir secciones vacias.
- **4.2** Actualizar oportunidad (stage_id, probability, expected_revenue)
- **4.3** Crear actividad de seguimiento (mail.activity con fecha y responsable)

## RULES

- **NEVER** inventes informacion que no este en las notas
- **NEVER** actualices Odoo sin aprobacion explicita del usuario
- Las acciones deben tener responsable Y fecha limite
- El email de seguimiento debe enviarse en las 2h siguientes a la llamada
- Si las notas son escuetas, generar resumen mas corto pero igualmente estructurado
- La actualizacion CRM es una SUGERENCIA. El comercial decide
- Si la llamada fue negativa, sugerir acciones de recuperacion (no solo "perdida")

## GOTCHAS

1. Las notas del usuario pueden ser desordenadas — estructurar sin perder matices
2. No confundir etapa actual de la oportunidad con la sugerida post-llamada
3. El email debe ser texto plano (no HTML) — los comerciales lo copian a Gmail
4. Al publicar en el chatter, usar subtype_xmlid "mail.mt_note" (nota interna, NO email a contactos)
5. Siempre verificar que el lead_id existe antes de escribir en Odoo
6. La nota a Odoo es el RESUMEN COMPLETO (5 secciones), no solo la seccion 4
7. El chatter NO carga CSS externo ni clases `iw-*`: usar SIEMPRE estilos inline (la plantilla ya los trae). Montserrat puede no estar en el backend — el font-stack incluye `Verdana, sans-serif` como fallback
8. El email (seccion 3) se redacta en texto plano para Gmail, PERO dentro del HTML del chatter se inserta con `<br>` en su caja `iw-callout`

## Delegates to
| Skill | Suite | When |
|-------|-------|------|
| odoo-pilot | common | Fetch/update CRM data (crm.lead, mail.message, mail.activity) |

## Consumed by
No other skill consumes call-summary directly.

## References

| File | Content |
|------|---------|
| [references/output_templates.md](references/output_templates.md) | Bloque de revision (texto) + plantilla HTML Indaws para el chatter + reglas del email |
