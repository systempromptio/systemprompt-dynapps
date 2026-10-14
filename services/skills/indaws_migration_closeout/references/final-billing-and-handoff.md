# Factura final / liquidación + Transición a mantenimiento

## Factura final (liquidación)

La migración se factura por **horas reales ejecutadas**, no por el estimado de
Fase 1 (ese sale.order era una estimación). Se crea vía la skill `odoo-pilot`
(scripts `auth.sh`, `create_record.sh`, `execute_method.sh`; nunca código ad-hoc).

### Datos de la factura (account.move, type out_invoice)
- **Cliente de facturación**: **Dynapps** (contacto Bélgica / Janik). El
  destinatario concreto lo confirma el usuario; dejarlo parametrizable.
- **Línea**: "migración" + nombre del proyecto en la descripción.
- **Cantidad** = **horas reales** del post-mortem (ver `postmortem-kpis.md`).
- **Precio unitario** = **45**.
- Origen / source: "migraciones con Dynapps".
- **Adjunto**: post-mortem/KPIs y referencia al documento de aceptación.

### Estado y envío
- Dejar la factura **en BORRADOR** (draft). **NO confirmar/validar** automáticamente.
- **REGLA DURA: NUNCA enviar correo desde Odoo.** No usar "Enviar por email". El
  envío a Dynapps es manual y fuera de Odoo, aunque el usuario diga "mándasela":
  significa que la enviará él, no el skill.

### Relación con el presupuesto de Fase 1
- Si el sale.order de Fase 1 sigue en borrador y Dynapps aceptó, puede confirmarse
  como parte del flujo (Fase 2/3), pero la **liquidación** factura lo realmente
  ejecutado. Reflejar el desvío en la descripción si difiere del presupuesto.

## Transición a soporte / mantenimiento

- Cerrar el `project.project` de la migración (estado cerrado/finalizado) vía
  `odoo-pilot` una vez entregado el handover y aceptado por el cliente.
- Definir el **traspaso a soporte**:
  - Contacto de soporte y alcance de **correcciones post-go-live** (qué cubre y qué
    pasa a presupuesto nuevo).
  - Repo GitLab y rama de mantenimiento de referencia.
  - Periodo de garantía/correcciones acordado con Dynapps, si lo hay.
- El go-live / copia de config a producción es de `odoo-copy-staging-master`; aquí
  solo se confirma que se ejecutó antes de cerrar.

## Conexión Odoo
Patrón de conexión y límites: `references/shared/odoo-connection.md`.
Autenticar primero (`eval $(./scripts/auth.sh)`). Incluir `context: {lang: es_ES}`.
