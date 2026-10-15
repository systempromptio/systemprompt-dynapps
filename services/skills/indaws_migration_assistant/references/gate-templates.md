# Plantillas de Validación de Gates — Copy-Paste

> **Formato visual de consulta.** Los entregables EXACTOS de cada gate (y sus conteos) son los de `scripts/migration_spec.json` → `gates.internal.{0..7}.checklist` y `gates.upgrade_lab.{A,B,C}.checklist`. Si cambian los entregables, edita el spec y refleja aquí el formato.
> Cada Gate tiene una plantilla de validación lista para copiar y rellenar.
> El responsable de ejecutar el Gate copia la plantilla, completa los campos,
> marca los entregables y la envía al aprobador para su firma.

---

## Gate 0 — Alcance y Propuesta Comercial

**Ejecuta:** PM | **Aprueba:** COO

```
╔══════════════════════════════════════════════════════════╗
║  GATE 0 — ALCANCE Y PROPUESTA COMERCIAL                ║
╚══════════════════════════════════════════════════════════╝

Proyecto:       _______________________________________________
Cliente:        _______________________________________________
Versión:        v_____ → v_____
Fecha:          _____ / _____ / _____
Ejecuta:        PM — ____________________________________________

┌─────────────────────────────────────────────────────────┐
│ ENTREGABLES                                    OK  N/A  │
├─────────────────────────────────────────────────────────┤
│ [ ] Estrategia decidida (1:1 / Refactoring)             │
│ [ ] Inventario de módulos custom completado             │
│ [ ] Reunión técnica de análisis realizada               │
│ [ ] Estimación de horas revisada por TL                 │
│ [ ] Integraciones externas identificadas                │
│ [ ] Propuesta comercial revisada (BC + PM + TL)         │
│ [ ] Condiciones guardias/fines de semana incluidas      │
│ [ ] Comunicación al cliente del proceso realizada       │
│ [ ] Firma del cliente aceptando condiciones             │
│ [ ] Documento de alcance detallado                      │
└─────────────────────────────────────────────────────────┘

Observaciones:
______________________________________________________________
______________________________________________________________

┌─────────────────────────────────────────────────────────┐
│ DECISIÓN:    [ ] APROBADO     [ ] NO APROBADO           │
│                                                         │
│ Aprueba:     COO — ___________________  Fecha: ___/___  │
│ Firma:       _____________________________________________│
│                                                         │
│ Si NO APROBADO, motivo:                                 │
│ ________________________________________________________│
└─────────────────────────────────────────────────────────┘
```

---

## Gate 1 — Análisis Técnico

**Ejecuta:** Dev | **Aprueba:** Tech Lead

```
╔══════════════════════════════════════════════════════════╗
║  GATE 1 — ANÁLISIS TÉCNICO                             ║
╚══════════════════════════════════════════════════════════╝

Proyecto:       _______________________________________________
Cliente:        _______________________________________________
Versión:        v_____ → v_____
Fecha:          _____ / _____ / _____
Ejecuta:        Dev — ____________________________________________

┌─────────────────────────────────────────────────────────┐
│ ENTREGABLES                                    OK  N/A  │
├─────────────────────────────────────────────────────────┤
│ [ ] Análisis de código completado                       │
│     - [ ] APIs deprecadas identificadas                 │
│     - [ ] Vulnerabilidades de seguridad revisadas       │
│     - [ ] Problemas de rendimiento detectados           │
│ [ ] Todos los módulos clasificados (Std/Studio/Core/Cus)│
│ [ ] Flujos críticos del cliente documentados (por BC)   │
│ [ ] Rama release/migration-X.Y creada                   │
│ [ ] Entorno de testing configurado y funcional          │
│ [ ] Ficha de Infraestructura del Servidor Destino       │
│     completada (ver sección 5)                          │
└─────────────────────────────────────────────────────────┘

Resumen del análisis:
  Módulos total: _____   Std: ___  Studio: ___  Core: ___  Custom: ___
  Issues encontrados: APIs deprecadas: ___  Seguridad: ___  Rendimiento: ___
  Horas estimadas código: _____h
  Horas estimadas BD: _____h

Observaciones:
______________________________________________________________
______________________________________________________________

┌─────────────────────────────────────────────────────────┐
│ DECISIÓN:    [ ] APROBADO     [ ] NO APROBADO           │
│                                                         │
│ Aprueba:     Tech Lead — _______________  Fecha: ___/___│
│ Firma:       _____________________________________________│
│                                                         │
│ Si NO APROBADO, motivo:                                 │
│ ________________________________________________________│
└─────────────────────────────────────────────────────────┘
```

---

## Gate 2 — Código Migrado

**Ejecuta:** Dev | **Aprueba:** Tech Lead

```
╔══════════════════════════════════════════════════════════╗
║  GATE 2 — CÓDIGO MIGRADO                               ║
╚══════════════════════════════════════════════════════════╝

Proyecto:       _______________________________________________
Cliente:        _______________________________________________
Versión:        v_____ → v_____
Fecha:          _____ / _____ / _____
Ejecuta:        Dev — ____________________________________________

┌─────────────────────────────────────────────────────────┐
│ ENTREGABLES                                    OK  N/A  │
├─────────────────────────────────────────────────────────┤
│ [ ] Todos los módulos migrados y funcionando            │
│ [ ] Pipeline CI/CD completamente verde                  │
│     - [ ] Pre-commit hooks (Ruff, Pylint-Odoo)          │
│     - [ ] ESLint + Prettier                             │
│     - [ ] Tests automáticos pasando                     │
│ [ ] Coverage ≥ 80% en módulos críticos                  │
│ [ ] Code review aprobado por Tech Lead                  │
│ [ ] Tests unitarios por módulo migrado                  │
└─────────────────────────────────────────────────────────┘

Progreso de módulos:
  Total: _____  Migrados: _____  Pendientes: _____
  Horas consumidas: _____h / _____h presupuestadas

Observaciones:
______________________________________________________________
______________________________________________________________

┌─────────────────────────────────────────────────────────┐
│ DECISIÓN:    [ ] APROBADO     [ ] NO APROBADO           │
│                                                         │
│ Aprueba:     Tech Lead — _______________  Fecha: ___/___│
│ Firma:       _____________________________________________│
│                                                         │
│ Si NO APROBADO, motivo:                                 │
│ ________________________________________________________│
└─────────────────────────────────────────────────────────┘
```

---

## Gate 3 — Base de Datos Migrada

**Ejecuta:** Dev | **Aprueba:** Tech Lead

```
╔══════════════════════════════════════════════════════════╗
║  GATE 3 — BASE DE DATOS MIGRADA                        ║
╚══════════════════════════════════════════════════════════╝

Proyecto:       _______________________________________________
Cliente:        _______________________________________________
Versión:        v_____ → v_____
Fecha:          _____ / _____ / _____
Ejecuta:        Dev — ____________________________________________

┌─────────────────────────────────────────────────────────┐
│ ENTREGABLES                                    OK  N/A  │
├─────────────────────────────────────────────────────────┤
│ [ ] BD migrada correctamente en entorno testing         │
│ [ ] Scripts pre-migrate documentados y versionados      │
│ [ ] Scripts post-migrate documentados y versionados     │
│ [ ] Validación de datos completada:                     │
│     - [ ] Conteo registros antes/después OK             │
│     - [ ] Relaciones M2O/O2M/M2M validadas             │
│     - [ ] Adjuntos y ficheros binarios verificados      │
│     - [ ] Saldos contables cuadrados                    │
│ [ ] Log de migración sin errores críticos               │
└─────────────────────────────────────────────────────────┘

Datos de la migración:
  Iteración nº: ___ de ___
  Tamaño BD: _____GB
  Tiempo de ejecución upgrade: _____h _____min
  Scripts pre-migrate: _____ ejecutados
  Scripts post-migrate: _____ ejecutados
  Errores encontrados: _____  Resueltos: _____

Observaciones:
______________________________________________________________
______________________________________________________________

┌─────────────────────────────────────────────────────────┐
│ DECISIÓN:    [ ] APROBADO     [ ] NO APROBADO           │
│                                                         │
│ Aprueba:     Tech Lead — _______________  Fecha: ___/___│
│ Firma:       _____________________________________________│
│                                                         │
│ Si NO APROBADO, motivo:                                 │
│ ________________________________________________________│
└─────────────────────────────────────────────────────────┘
```

---

## Gate 4 — Testing Funcional (FASE CRÍTICA)

**Ejecuta:** BC + Dev | **Aprueba:** Tech Lead

```
╔══════════════════════════════════════════════════════════╗
║  GATE 4 — TESTING FUNCIONAL (FASE CRÍTICA)             ║
╚══════════════════════════════════════════════════════════╝

Proyecto:       _______________________________________________
Cliente:        _______________________________________________
Versión:        v_____ → v_____
Fecha:          _____ / _____ / _____
Ejecuta:        BC — ________________  Dev — ________________

┌─────────────────────────────────────────────────────────┐
│ CHECKLIST FUNCIONAL (9 items)                           │
├─────────────────────────────────────────────────────────┤
│ [ ] 1. Venta completa (presupuesto→pedido→factura→cobro)│
│ [ ] 2. Compra completa (solicitud→pedido→recepción→     │
│        factura→pago)                                    │
│ [ ] 3. Inventario (transferencia + reabastecimiento)    │
│ [ ] 4. Fabricación: BOM→Orden→Producción→Entrega   N/A?│
│ [ ] 5. Asiento manual + Conciliación bancaria           │
│ [ ] 6. Nóminas: cálculo y generación              N/A? │
│ [ ] 7. Integraciones externas (API, import/export)      │
│ [ ] 8. Informes clave del cliente (PDF, Excel)          │
│ [ ] 9. Flujos adicionales específicos: ____________     │
├─────────────────────────────────────────────────────────┤
│ CHECKLIST TÉCNICO (7 items)                             │
├─────────────────────────────────────────────────────────┤
│ [ ] 1. Módulos/logs/rendimiento < 3s OK                 │
│ [ ] 2. Vistas y componentes JS/OWL cargan sin errores   │
│ [ ] 3. Cron jobs ejecutan correctamente                 │
│ [ ] 4. Informes generan PDF correctamente               │
│ [ ] 5. Integraciones externas responden OK              │
│ [ ] 6. Emails/notificaciones envían OK                  │
│ [ ] 7. Permisos y grupos de seguridad correctos         │
│ (infra: PG/backup/disco → Ficha Fase 1 + Validación F5) │
├─────────────────────────────────────────────────────────┤
│ CRITERIOS GO/NO-GO                                      │
├─────────────────────────────────────────────────────────┤
│ [ ] 100% flujos críticos OK                             │
│ [ ] 100% checklist técnico OK                           │
│ [ ] Cliente firma acta de conformidad                   │
│                                                         │
│ Resultado:  [ ] GO    [ ] NO-GO                         │
│ Si NO-GO, motivo: _____________________________________│
└─────────────────────────────────────────────────────────┘

Incidencias KO encontradas: _____  Resueltas: _____

Observaciones:
______________________________________________________________
______________________________________________________________

┌─────────────────────────────────────────────────────────┐
│ ACTA DE CONFORMIDAD                                     │
│                                                         │
│ BC:       ___________________  Firma: _____  Fecha: ___ │
│ TL:       ___________________  Firma: _____  Fecha: ___ │
│ Cliente:  ___________________  Firma: _____  Fecha: ___ │
├─────────────────────────────────────────────────────────┤
│ DECISIÓN:    [ ] APROBADO     [ ] NO APROBADO           │
│                                                         │
│ Aprueba:     Tech Lead — _______________  Fecha: ___/___│
│ Firma:       _____________________________________________│
│                                                         │
│ Si NO APROBADO, motivo:                                 │
│ ________________________________________________________│
└─────────────────────────────────────────────────────────┘
```

---

## Gate 5 — Ensayo General (Rehearsal)

**Ejecuta:** Dev | **Aprueba:** Tech Lead

```
╔══════════════════════════════════════════════════════════╗
║  GATE 5 — ENSAYO GENERAL (REHEARSAL)                   ║
╚══════════════════════════════════════════════════════════╝

Proyecto:       _______________________________________________
Cliente:        _______________________________________________
Versión:        v_____ → v_____
Fecha:          _____ / _____ / _____
Ejecuta:        Dev — ____________________________________________

┌─────────────────────────────────────────────────────────┐
│ ENTREGABLES                                    OK  N/A  │
├─────────────────────────────────────────────────────────┤
│ [ ] Rehearsal ejecutado sin errores bloqueantes         │
│ [ ] Tiempos medidos y dentro de ventana acordada        │
│ [ ] Plan de rollback documentado y probado              │
│ [ ] Smoke tests automatizados funcionando               │
│ [ ] Checklist Infraestructura 100% completado y firmado │
│     (ver plantilla de infraestructura en sección 9)     │
└─────────────────────────────────────────────────────────┘

Tiempos medidos del rehearsal:
  ┌──────────────────────────────────┬───────────┐
  │ Paso                             │ Duración  │
  ├──────────────────────────────────┼───────────┤
  │ Backup de BD                     │ ___h ___m │
  │ Cierre accesos usuarios          │ ___h ___m │
  │ Ejecución upgrade                │ ___h ___m │
  │ Scripts post-migrate             │ ___h ___m │
  │ Smoke tests                      │ ___h ___m │
  │ Apertura accesos                 │ ___h ___m │
  ├──────────────────────────────────┼───────────┤
  │ TOTAL                            │ ___h ___m │
  └──────────────────────────────────┴───────────┘

  Ventana de mantenimiento acordada: _____h
  ¿Cabe en ventana?  [ ] Sí    [ ] No (requiere ajuste)

Observaciones:
______________________________________________________________
______________________________________________________________

┌─────────────────────────────────────────────────────────┐
│ FIRMAS DE INFRAESTRUCTURA                               │
│                                                         │
│ Developer:   ___________________  Fecha: ___/___        │
│ Tech Lead:   ___________________  Fecha: ___/___        │
├─────────────────────────────────────────────────────────┤
│ DECISIÓN:    [ ] APROBADO     [ ] NO APROBADO           │
│                                                         │
│ Aprueba:     Tech Lead — _______________  Fecha: ___/___│
│ Firma:       _____________________________________________│
│                                                         │
│ Si NO APROBADO, motivo:                                 │
│ ________________________________________________________│
└─────────────────────────────────────────────────────────┘
```

---

## Gate 6 — Puesta en Producción (Go-Live)

**Ejecuta:** BC + Dev | **Aprueba:** Tech Lead + CTO Ops

```
╔══════════════════════════════════════════════════════════╗
║  GATE 6 — PUESTA EN PRODUCCIÓN (GO-LIVE)               ║
╚══════════════════════════════════════════════════════════╝

Proyecto:       _______________________________________________
Cliente:        _______________________________________________
Versión:        v_____ → v_____
Fecha Go-Live:  _____ / _____ / _____
Ejecuta:        BC — ________________  Dev — ________________

┌─────────────────────────────────────────────────────────┐
│ PRERREQUISITOS (todos deben ser OK)                     │
├─────────────────────────────────────────────────────────┤
│ [ ] Checklist funcional Fase 4 — todos OK               │
│ [ ] Checklist técnico Fase 4 — 7 items OK               │
│ [ ] Checklist infraestructura Fase 5 — firmado          │
│ [ ] Rehearsal Fase 5 — tiempos dentro de ventana        │
├─────────────────────────────────────────────────────────┤
│ PROTOCOLO GO-LIVE                                       │
├─────────────────────────────────────────────────────────┤
│ [ ] Plan aprobado por TL + PM                           │
│ [ ] Ventana comunicada al cliente (≥ 1 semana antes)    │
│ [ ] Equipo guardia confirmado (Dev + BC + PM)           │
│ [ ] Protocolo de comunicación/escalado definido         │
│ [ ] Backup producción actual (dump + filestore) en      │
│     almacenamiento seguro y verificado restaurable      │
├─────────────────────────────────────────────────────────┤
│ EJECUCIÓN                                               │
├─────────────────────────────────────────────────────────┤
│ [ ] Accesos usuarios cerrados                           │
│ [ ] Freeze de configuración/código activo               │
│ [ ] Migración ejecutada (pasos de rehearsal)            │
│ [ ] Smoke tests post-deploy OK (15-30 min)              │
│ [ ] Backup post-migración (punto cero) realizado        │
│ [ ] Apertura gradual de accesos                         │
├─────────────────────────────────────────────────────────┤
│ POST GO-LIVE                                            │
├─────────────────────────────────────────────────────────┤
│ [ ] Monitorización 48h activada                         │
│ [ ] Cliente confirma operación normal                   │
└─────────────────────────────────────────────────────────┘

Hora inicio: _____  Hora fin: _____  Duración total: _____

Observaciones:
______________________________________________________________
______________________________________________________________

┌─────────────────────────────────────────────────────────┐
│ DECISIÓN:    [ ] APROBADO     [ ] NO APROBADO           │
│                                                         │
│ Aprueba:     Tech Lead — _______________  Fecha: ___/___│
│              CTO Ops — ________________  Fecha: ___/___ │
│ Firma TL:    _____________________________________________│
│ Firma CTO:   _____________________________________________│
│                                                         │
│ Si NO APROBADO, motivo:                                 │
│ ________________________________________________________│
└─────────────────────────────────────────────────────────┘
```

---

## Gate 7 — Cierre del Proyecto

**Ejecuta:** PM + TL + BC | **Aprueba:** COO

```
╔══════════════════════════════════════════════════════════╗
║  GATE 7 — CIERRE DEL PROYECTO                          ║
╚══════════════════════════════════════════════════════════╝

Proyecto:       _______________________________________________
Cliente:        _______________________________________________
Versión:        v_____ → v_____
Fecha:          _____ / _____ / _____
Ejecuta:        PM — _________  TL — _________  BC — _________

┌─────────────────────────────────────────────────────────┐
│ ENTREGABLES                                    OK  N/A  │
├─────────────────────────────────────────────────────────┤
│ [ ] Período soporte 2 semanas completado                │
│ [ ] Todas las incidencias post-migración resueltas      │
│ [ ] Retrospectiva realizada con todo el equipo          │
│     - [ ] Qué funcionó bien documentado                 │
│     - [ ] Qué no funcionó bien documentado              │
│     - [ ] Lecciones aprendidas registradas              │
│     - [ ] Mejoras propuestas para futuros proyectos     │
│ [ ] Guía de migración actualizada con aprendizajes      │
│ [ ] Cierre formal comunicado al cliente                 │
└─────────────────────────────────────────────────────────┘

Resumen del proyecto:
  Duración total:              _____ semanas
  Horas presupuestadas:        _____h
  Horas consumidas:            _____h
  Desviación:                  _____% (sobre/bajo presupuesto)
  Incidencias post-migración:  _____
  Tiempo medio resolución:     _____h

Observaciones:
______________________________________________________________
______________________________________________________________

┌─────────────────────────────────────────────────────────┐
│ DECISIÓN:    [ ] APROBADO     [ ] NO APROBADO           │
│                                                         │
│ Aprueba:     COO — ___________________  Fecha: ___/___  │
│ Firma:       _____________________________________________│
│                                                         │
│ Si NO APROBADO, motivo:                                 │
│ ________________________________________________________│
└─────────────────────────────────────────────────────────┘
```

---

## Referencia rápida — Ejecuta / Aprueba

| Gate | Ejecuta | Aprueba |
|------|---------|---------|
| Gate 0 | PM | COO |
| Gate 1 | Dev | Tech Lead |
| Gate 2 | Dev | Tech Lead |
| Gate 3 | Dev | Tech Lead |
| Gate 4 | BC + Dev | Tech Lead |
| Gate 5 | Dev | Tech Lead |
| Gate 6 | BC + Dev | Tech Lead + CTO Ops |
| Gate 7 | PM + TL + BC | COO |
