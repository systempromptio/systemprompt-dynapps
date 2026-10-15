# Ghost Follow-up — Actividades Complementarias No Email

## Concepto

Las actividades "ghost" son acciones de seguimiento complementarias al email que NO se envían por correo. Se ejecutan en otros canales (LinkedIn, teléfono, WhatsApp) para crear múltiples puntos de contacto con el lead sin saturar su inbox.

Se crean como `mail.activity` en Odoo, vinculadas al contacto (`res.partner`) y asignadas al comercial responsable.

## Tipos de Actividades Ghost

### 1. LinkedIn Connect
- **Descripción:** Enviar solicitud de conexión en LinkedIn al contacto
- **Cuándo:** Después del primer o segundo email de la secuencia
- **Nota para el comercial:** "Enviar solicitud de conexión en LinkedIn. Personalizar el mensaje mencionando [tema de la campaña]. No mencionar el email directamente — que parezca orgánico."
- **Tipo Odoo:** To-Do (`mail.mail_activity_data_todo`)
- **Icono sugerido:** 🔗

### 2. LinkedIn Comment
- **Descripción:** Comentar la última publicación del contacto en LinkedIn
- **Cuándo:** 1-2 días después de un email educativo o de valor
- **Nota para el comercial:** "Buscar la última publicación del contacto en LinkedIn y dejar un comentario relevante y genuino. El objetivo es generar visibilidad, NO vender."
- **Tipo Odoo:** To-Do (`mail.mail_activity_data_todo`)
- **Icono sugerido:** 💬

### 3. Phone Call
- **Descripción:** Llamada de seguimiento al contacto
- **Cuándo:** Después del email de CTA principal o después de que el contacto haya abierto varios emails
- **Nota para el comercial:** "Llamada de seguimiento. Mencionar brevemente el contenido enviado por email ('no sé si tuviste oportunidad de ver...'). Ofrecer resolver dudas y proponer siguiente paso."
- **Tipo Odoo:** Phone Call (`mail.mail_activity_data_call`)
- **Icono sugerido:** 📞

### 4. WhatsApp Resource
- **Descripción:** Enviar un recurso personalizado por WhatsApp
- **Cuándo:** Después de que el contacto haya mostrado interés (aperturas, clicks) o si hay relación previa
- **Nota para el comercial:** "Enviar por WhatsApp un recurso relevante (caso de éxito, guía, infografía). Tono informal pero profesional. Ejemplo: 'Hola [nombre], vi que te podría interesar esto — [link]. Si tienes 5 min te lo comento 😊'"
- **Tipo Odoo:** To-Do (`mail.mail_activity_data_todo`)
- **Icono sugerido:** 📱
- **⚠️ Solo si hay relación previa.** No enviar WhatsApp en frío.

### 5. Meeting Proposal
- **Descripción:** Proponer reunión breve para profundizar
- **Cuándo:** Fase final de la secuencia, cuando el lead ha mostrado engagement
- **Nota para el comercial:** "Proponer reunión de 15-20 min. Enfoque: 'Me gustaría enseñarte cómo funciona [solución] aplicado a tu caso concreto.' Sin presión de venta."
- **Tipo Odoo:** Meeting (`mail.mail_activity_data_meeting`)
- **Icono sugerido:** 📅

## Cuándo Usar Ghost Follow-up

| Tipo de Campaña | Ghost Recomendado | Actividades Sugeridas |
|---|---|---|
| Lead Nurturing | ✅ Sí | LinkedIn Connect + Phone Call |
| Reactivación | ✅ Sí (fuerte) | LinkedIn Connect + Phone Call + WhatsApp |
| Lead Generation | ✅ Sí | LinkedIn Connect + Phone Call |
| Producto/Feature | ⚡ Opcional | Phone Call (si hay interés) |
| Evento | ⚡ Opcional | LinkedIn Connect (speakers/asistentes) |
| Newsletter | ❌ No | — |

## Timing Recomendado

```
Email 1 ──────── Email 2 ──────── Email 3 ──────── Email 4
     │                │                │
     └─ LinkedIn    └─ LinkedIn     └─ Phone Call
        Connect        Comment          o WhatsApp
        (+1 día)       (+1 día)         (+1 día)
```

**Regla general:** La actividad ghost se programa 1 día después del email correspondiente, para que el comercial la ejecute cuando el contacto ya ha tenido tiempo de leer el email.

## Script de Creación

Usar `scripts/create_followup_activities.sh`:

```bash
# Ejemplo: Crear actividad LinkedIn Connect para 3 contactos
bash scripts/create_followup_activities.sh \
  --partner-ids "45,67,89" \
  --activity-type "linkedin_connect" \
  --summary "Conectar en LinkedIn — Campaña Nurturing Q1" \
  --user-id 2 \
  --date-deadline "2025-02-20"
```

### Tipos disponibles en el script:
- `linkedin_connect` — Conexión LinkedIn
- `linkedin_comment` — Comentar en LinkedIn
- `phone_call` — Llamada de seguimiento
- `whatsapp` — Enviar recurso por WhatsApp
- `meeting` — Proponer reunión
- `custom` — Actividad personalizada

## Buenas Prácticas

1. **No saturar:** Máximo 2-3 actividades ghost por campaña por contacto
2. **Timing natural:** Espaciar las actividades para que no parezcan coordinadas
3. **Personalización real:** Cada actividad debe sentirse genuina, no automatizada
4. **Registrar resultado:** El comercial debe actualizar la actividad en Odoo con el resultado
5. **Respetar el "no":** Si el contacto no responde a 2 intentos ghost, no insistir
