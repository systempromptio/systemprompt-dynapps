# Fases de Migración Odoo — Proceso con Gates Obligatorios

> Fuente: Artículo 2602 — Guía de Migración Odoo (Indaws/Dynapps)
> IMPORTANTE: Cada fase tiene un Gate obligatorio. No se avanza sin aprobación.

---

## Conceptos Clave

### Traspaso vs Migración
- **Traspaso:** Mover entorno Odoo de un servidor a otro (sin cambio de versión). ~15h.
- **Migración:** Cambio de versión Odoo. Puede incluir traspaso o realizarse en el mismo servidor.

### Estrategias de Migración

**Estrategia 1:1 (Migración Directa):**
- Migrar cada módulo custom tal cual, adaptando solo APIs a la nueva versión
- Default: todo es Custom
- Solo se marca Standard si el módulo replica 100% funcionalidad nativa
- Categorías Core y Studio NO se usan en esta estrategia

**Estrategia Refactoring (Optimización):**
- Reducir deuda técnica durante la migración
- Reclasificar cada modulo con `odoo-classification-guide` (escalera Standard > Core > Studio > Custom; criterio de codigo gobernado)
- Si un módulo Core resuelve la funcionalidad, es obligatorio usarlo
- Requiere más análisis pero reduce mantenimiento futuro

---

## Fase 0 — Kick-off, Alcance y Propuesta Comercial

### Actividades
1. **Definir estrategia de migración:** 1:1 vs Refactoring. Decisión conjunta con Tech Lead.
2. **Inventario de módulos custom:** Escanear ficheros, clasificación inicial por tipo y dependencias.
3. **Reunión de análisis técnico:** Obligatoria con equipo técnico tras inventario. Resolver dudas, buscar soluciones conjuntas, acordar estrategia por módulo ANTES de estimar.
4. **Estimación de horas con Claude Code + revisión humana** (ver `estimation.md` para fórmulas detalladas).
5. **Identificar integraciones externas:** APIs, imports/exports, webhooks, servicios de terceros.
6. **Propuesta comercial revisada conjuntamente:** Revisión obligatoria por Consultor + Comercial/PM + Tech Lead.
7. **Inclusión obligatoria en propuesta:** Condiciones de trabajo en fin de semana, guardia técnica, horas de soporte post-migración.
8. **Comunicación clara al cliente:** Explicar proceso completo, paso a estándar/studio, plazos estimados y ventanas de mantenimiento. El desarrollo a medida que quede se entrega **gobernado** (revisado, testeable, migrable), para que las futuras migraciones sean predecibles.

### Gate 0: Propuesta económica revisada + Documento de alcance firmado
- **Aprueba:** COO
- **Ejecuta:** PM
- **Entregables:** Propuesta comercial revisada y aprobada, documento de alcance detallado, firma del cliente aceptando condiciones.

---

## Fase 1 — Preparación Técnica

### Actividades
1. **Análisis de código:**
   - APIs deprecadas (@api.one, @api.returns, @api.cr, @api.model_cr, etc.)
   - Vulnerabilidades de seguridad (SQL injection, abuso de sudo, eval/exec)
   - Problemas de rendimiento (N+1 queries, search sin limit, búsquedas ineficientes)
2. **Clasificación de módulos:** Asignar etiqueta Standard / Studio / Core / Custom según estrategia de Fase 0.
3. **Documentar flujos críticos del cliente** (responsabilidad BC): Ventas, Compras, Inventario, Fabricación (si aplica), Contabilidad, RRHH/Nóminas (si aplica), Integraciones externas.
4. **Creación de rama:** `release/migration-X.Y` per gitflow.
5. **Configurar entorno de testing:** Preparar entorno aislado para pruebas de migración.

### Formato de Tareas de Migración (tabla por módulo)

| Nombre del módulo | Estrategia (Etiqueta) | Notas | Hecho |
|---|---|---|---|
| partner_extended_info | Studio | Replicar campos custom en Studio. No migrar wizards antiguos. | [ ] |
| indaws_sale_purchase | Core | Usar módulo OCA sale_purchase_stock. Añadir campos custom, script migración y 2 acciones automatizadas. | [ ] |
| custom_sale_order | Custom | Migrar solo modelo sale.order. NO migrar vistas partner_views.xml (usar estándar). | [ ] |

### Ficha de Infraestructura del Servidor Destino (OBLIGATORIA)

Completar ANTES de Gate 1. Detectar problemas de infra antes de invertir horas de desarrollo.

| # | Dato | Valor | Notas |
|---|---|---|---|
| 1 | Tipo hosting destino | | Cloud Indaws / Odoo.sh / Otro |
| 2 | Servidor destino: SO | | ej. Ubuntu 22.04 LTS |
| 3 | Servidor destino: Versión PostgreSQL | | `psql --version` |
| 4 | Servidor destino: Disco disponible | | `df -h /` |
| 5 | Servidor destino: RAM | | `free -h` |
| 6 | Servidor destino: CPU | | `nproc` o cores |
| 7 | Entorno origen: Versión PostgreSQL | | Versión con la que se creó el dump |
| 8 | Backup: Tamaño comprimido (.zip) | | Tamaño fichero .zip descargado |
| 9 | Backup: Tamaño BD descomprimida | | Tamaño dump.sql descomprimido |
| 10 | Backup: Tamaño filestore | | Tamaño directorio filestore |
| 11 | Conectividad: Acceso SSH | | Verificar usuario + clave |
| 12 | Conectividad: Acceso S3/bucket | | Para descarga/subida de backup |
| 13 | Conectividad: Puertos abiertos | | 8069, 8072, 443 |
| 14 | Conectividad: VPN requerida | | Sí/No — datos de conexión |

**REGLAS CRÍTICAS DE INFRAESTRUCTURA:**
1. PostgreSQL destino DEBE ser >= versión del dump. Un dump PostgreSQL 18 NO puede restaurarse en PostgreSQL 15.
2. Espacio libre en disco >= 4x tamaño del backup comprimido.

### Gate 1: Análisis técnico completado y revisado por Tech Lead
- **Aprueba:** Tech Lead
- **Ejecuta:** Dev
- **Entregables:** Análisis de código completo, módulos clasificados, flujos críticos documentados, rama de migración creada, entorno de testing funcional, Ficha de Infraestructura completada.

---

## Fase 2 — Migración de Código

### Actividades
1. **Migración de módulos en orden de dependencias:** Empezar por módulos base sin dependencias, avanzar a dependientes.
2. **Aplicar patrones documentados:** XML (Vistas, acciones, menús, datos, seguridad), Python (ORM, campos compute, onchange, constraints), JavaScript/OWL (especialmente crítico en v16+), SCSS.
3. **Tests unitarios por módulo migrado.**
4. **CI/CD en verde:** Pipeline completo: Pre-commit hooks (Ruff, Pylint-Odoo, ESLint, Prettier), tests automáticos, cobertura >= 80% en módulos críticos.

### Gate 2: Todos los módulos migrados, pipeline verde, code review aprobado
- **Aprueba:** Tech Lead
- **Ejecuta:** Dev
- **Entregables:** Todos los módulos migrados y funcionando, pipeline CI/CD completamente verde, code review aprobado por Tech Lead, tests unitarios pasando.

---

## Fase 3 — Migración de Base de Datos

### Iteraciones por Tamaño de Proyecto
- **Pequeño:** Test → Producción (2 iteraciones)
- **Mediano:** Test 1 → Ensayo → Producción (3 iteraciones; horas en `odoo-effort-estimator`)
- **Grande:** Test 1 → Test 2 → Ensayo → Producción (4 iteraciones, +20h)

### Actividades
1. **Ejecutar iteraciones de upgrade:** Servicio oficial de Odoo o scripts propietarios validados.
2. **Scripts de migración de datos custom:** Pre-migrate (antes del upgrade Odoo) y Post-migrate (después, limpieza/transformaciones).
3. **Validación de datos migrados:**
   - Conteo de registros antes/después por modelo crítico
   - Validación de relaciones (Many2one, One2many, Many2many)
   - Verificación de adjuntos y ficheros binarios
   - Validación de saldos contables y conciliaciones

### Gate 3: BD migrada en entorno de testing sin errores
- **Aprueba:** Tech Lead
- **Ejecuta:** Dev
- **Entregables:** BD correctamente migrada en testing, scripts pre/post-migrate documentados y versionados, validación de datos completada, log de migración sin errores críticos.

---

## Fase 4 — Testing Funcional en Pre-producción (FASE CRÍTICA)

> ⚠️ AVISO: Esta fase ha sido la principal causa de errores en producción en proyectos pasados. TODOS los checklists deben completarse antes de avanzar. No se avanza a producción sin firma de conformidad del cliente, Consultor y Tech Lead.

### Actividades
1. **Mapeo de flujos críticos del cliente** (ver `checklists.md` para detalle completo)
2. **Checklist funcional obligatorio** (9 items)
3. **Checklist técnico obligatorio** (7 items)
4. **Criterios Go/No-Go**

### Criterios Go/No-Go

**GO (Avanzar a Producción):**
- 100% flujos críticos funcionando correctamente (OK)
- 100% checklist técnico completado (OK)
- Cliente firma acta de conformidad

**NO-GO (Parar proceso):**
- Cualquier flujo crítico KO → Parar, corregir, re-testear completamente
- Más del 10% del checklist técnico KO
- Cliente no firma o reporta incidencias bloqueantes

### Gate 4: Acta de conformidad firmada por BC + TL + Cliente
- **Aprueba:** Tech Lead
- **Ejecuta:** Consultor + Dev
- **Entregables:** 100% checklist funcional completado, 100% checklist técnico completado, acta de conformidad firmada, todas las incidencias KO resueltas y re-validadas.

---

## Fase 5 — Ensayo General (Rehearsal)

### Actividades
1. **Simulación completa de producción:** Ejecutar el MISMO proceso exacto que en producción, pero en entorno de aceptación/staging.
2. **Medición de tiempos:** Cronometrar CADA paso: backup, cierre accesos, ejecución upgrade, scripts post-migrate, smoke tests, apertura accesos.
3. **Plan de rollback documentado y probado.**
4. **Validación de smoke tests automatizados.**

### Checklist de Validación de Infraestructura (OBLIGATORIO — BLOQUEANTE)

Ver `checklists.md` para el checklist completo (Variante A: Servidores propios, Variante B: Odoo.sh).

### Firmas de Validación

| Rol | Nombre | Fecha | Firma |
|---|---|---|---|
| Developer | | | |
| Tech Lead | | | |

### Gate 5: Ensayo exitoso, tiempos dentro de ventana planificada
- **Aprueba:** Tech Lead
- **Ejecuta:** Dev
- **Entregables:** Ensayo ejecutado sin errores bloqueantes, tiempos medidos y dentro de ventana, plan de rollback documentado y validado, smoke tests automatizados funcionando, Checklist de Infraestructura 100% completado y firmado.

---

## Fase 6 — Puesta en Producción (Go-Live)

> ⚠️ PROTOCOLO OBLIGATORIO: La planificación del Go-Live NUNCA será decidida únicamente por el consultor. Requiere aprobación conjunta de Tech Lead + PM (COO + CTO Ops si necesario).

### Prerrequisitos OBLIGATORIOS antes del Go-Live
- [ ] Checklist funcional FASE 4.2: Todos los flujos OK
- [ ] Checklist técnico FASE 4.3: Los 7 ítems OK
- [ ] Checklist infraestructura FASE 5: Todos los ítems OK + firmado
- [ ] Ensayo FASE 5: Tiempos medidos y dentro de ventana aceptable

### Protocolo Obligatorio de Go-Live
1. **Planificación aprobada por Tech Lead + PM:** NUNCA solo por consultor.
2. **Ventana de mantenimiento comunicada al cliente:** Mínimo 1 semana de antelación. Email formal.
3. **Equipo de guardia definido y confirmado:** Dev + Consultor + PM disponibles.
4. **Protocolo de comunicación definido:** Canales primarios, protocolo escalación, contactos de emergencia.
5. **Backup completo de producción actual:** dump.sql + filestore completo subido a almacenamiento seguro. Validar que el backup es restaurable ANTES del go-live.

### Ejecución del Go-Live
1. Cerrar acceso de usuarios
2. Congelación: prohibición absoluta de cambios de configuración o código desde el día antes
3. Ejecutar proceso de migración: seguir EXACTAMENTE los pasos validados en Ensayo
4. Smoke tests post-deploy de flujos críticos (15-30 min)
5. Backup post-migración (punto cero): ANTES de dar acceso a usuarios
6. Apertura gradual de acceso: primero usuarios clave/power users, luego el resto

### Monitorización Post Go-Live
- Monitorización activa 48h post-go-live
- Presencia prioritaria de técnico y consultor: disponibilidad inmediata

### Gate 6: Smoke tests OK + Cliente confirma operación normal
- **Aprueba:** Tech Lead + CTO Ops
- **Ejecuta:** Consultor + Dev
- **Entregables:** Smoke tests completados sin errores, cliente confirma operación normal, backup post-migración (punto cero) completado, equipo de monitorización activo.

---

## Fase 7 — Soporte Post-migración y Cierre

### Actividades
1. **Período de soporte intensivo:** 2 semanas post-go-live con tiempos de respuesta reducidos.
2. **Canal prioritario de incidencias:** Email/ticket prioritario, teléfono directo, canal Teams dedicado.
3. **Coordinación explícita entre equipos:** Daily standups primera semana.
4. **Plan de rollback de emergencia activo:** Mantener backups y plan disponibles 2 semanas.
5. **Retrospectiva del proyecto:** Reunión formal al final de 2 semanas.
6. **Actualizar esta guía:** Incorporar nuevos aprendizajes.

### Gate 7 (Cierre): Retrospectiva completada + Guía actualizada
- **Aprueba:** COO
- **Ejecuta:** PM + Tech Lead + Consultor
- **Entregables:** Período de soporte completado, retrospectiva documentada, guía actualizada, cierre formal del proyecto.

---

## Resumen — Quién Aprueba Cada Gate

| Gate | Accountable (Aprueba) | Responsible (Ejecuta) |
|---|---|---|
| Gate 0: Alcance + Propuesta | COO | PM |
| Gate 1: Análisis técnico | Tech Lead | Dev |
| Gate 2: Código migrado | Tech Lead | Dev |
| Gate 3: BD migrada | Tech Lead | Dev |
| Gate 4: Testing funcional OK | Tech Lead | Consultor + Dev |
| Gate 5: Ensayo exitoso | Tech Lead | Dev |
| Gate 6: Go-Live OK | Tech Lead + CTO Ops | Consultor + Dev |
| Gate 7: Cierre del proyecto | COO | PM + TL + Consultor |
