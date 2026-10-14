# Protocolo de Comunicación y Seguimiento

> Fuente: Artículo 2602 — Sección 3.1
> El desarrollo técnico (Fases 2 y 3) NO puede ser una caja negra.
> Todos los perfiles deben tener visibilidad del progreso y canales claros para resolver bloqueos.

---

## A. Frecuencia de Comunicación

| De → A | Frecuencia | Formato | Canal |
|---|---|---|---|
| Dev → BC | Por hito o bloqueo | Informe estructurado | Tarea Odoo |
| Dev → TL | Por hito o bloqueo | Actualización breve | Tarea Odoo o chat |
| TL → PM | Por hito o bloqueo | Resumen ejecutivo | Email / reunión |
| BC → Cliente | Por hito | Email formal | Email |
| Cualquiera | Bajo demanda | Alerta de bloqueo | Tarea + mención |

---

## B. Plantilla "Informe Dev → Consultoría" (por hito o bloqueo)

El Dev publica este informe como comentario en la tarea principal de migración:

| Sección | Contenido |
|---|---|
| **Fase actual** | [Fase X] — Progreso global: X/Y módulos (Z%) |
| **Completado esta semana** | Lista de módulos/tareas completados con detalle breve |
| **En progreso** | Módulos en progreso con % de completado estimado |
| **Bloqueado — requiere decisión BC** | Descripción del bloqueo + opciones propuestas por Dev |
| **Próxima semana** | Previsión de trabajo y objetivos |
| **Riesgos detectados** | Nuevos riesgos o cambios en riesgos existentes |

### Ejemplo de uso
```
=== INFORME Dev → BC (14-Feb-2026) ===

Fase actual: Fase 2 — Progreso global: 8/15 módulos (53%)

Completado esta semana:
- partner_extended_info (Studio): Campos replicados en Studio, tests OK
- indaws_sale_purchase (Core): OCA module configurado, 2 acciones automatizadas creadas

En progreso:
- custom_sale_order (Custom): 60% — Modelo sale.order migrado, faltan vistas
- custom_invoice_report (Custom): 30% — QWeb en progreso

Bloqueado — requiere decisión BC:
- Campo x_studio_margin_type: ¿Mantener cálculo custom o usar el nativo de v18?
  Opción A: Mantener custom (4h extra)
  Opción B: Migrar a nativo (2h, pero pierde informe X)

Próxima semana:
- Completar custom_sale_order + custom_invoice_report
- Iniciar módulos JS/OWL (custom_kanban_view)

Riesgos detectados:
- custom_kanban_view usa jQuery directo → OWL migration más costosa de lo estimado (+8h)
```

---

## C. Plantilla "Resumen TL → PM" (por hito o bloqueo)

| Sección | Contenido |
|---|---|
| **% progreso global** | Progreso general del proyecto |
| **Horas consumidas vs presupuesto** | X h consumidas / Y h presupuestadas (Z%) |
| **Desviaciones** | Desviaciones detectadas + plan de acción |
| **Bloqueos escalados** | Bloqueos que requieren intervención del PM |
| **Forecast fecha entrega** | Fecha estimada de finalización actualizada |

### Ejemplo de uso
```
=== RESUMEN TL → PM (Semana 3, 17-Feb-2026) ===

% progreso global: 53% (8/15 módulos completados)

Horas consumidas vs presupuesto: 85h / 180h (47%)
— Alineado con progreso.

Desviaciones:
- Módulo custom_kanban_view: +8h sobre estimación (jQuery → OWL)
  Plan: Absorber con margen de contingencia (18%)

Bloqueos escalados:
- Ninguno esta semana. BC respondió en <12h a todas las consultas.

Forecast fecha entrega: 14-Mar-2026 (sin cambios)
```

---

## D. Protocolo para Decisiones Funcionales Bloqueantes

1. **Dev documenta** el problema + opciones propuestas en la tarea Odoo.
2. **Dev menciona a BC** en la tarea. BC responde en < 24h laborables.
3. Si BC no responde en 24h → **TL escala a PM**.
4. Cada decisión se documenta en el log de la tarea con fecha, decisor y motivo.

### Formato de documentación de decisión
```
=== DECISIÓN FUNCIONAL ===
[DD-MMM] Decisión: [Qué se decidió]
         Decidido por: [Nombre, Rol]
         Motivo: [Por qué]
         Alternativa descartada: [Qué se descartó y por qué]
```

---

## Regla de Oro

> **Ningún perfil debe esperar más de 48h por una respuesta que bloquee su trabajo.**
> Si ocurre, se escala automáticamente al siguiente nivel.

### Cadena de escalación
```
Dev → BC (24h) → TL → PM (48h) → COO/CTO
```
