# Checklists de Migración

> **Esto es la referencia de METODOLOGÍA** (detalle humano de Fases 4 y 5: flujos, técnico, infraestructura, Go/No-Go).
> Los **checklists OPERATIVOS** que el motor crea como ítems nativos en las tareas de Odoo viven en `scripts/migration_spec.json` (`checklists.*` y `gates.*.checklist`) — esa es su fuente de verdad. El artículo 2602 es una copia de consulta y se mantiene a mano.
> TODOS los checklists deben completarse antes de avanzar a producción.

---

## 1. Mapeo de Flujos Críticos del Cliente (Fase 4.1)

Estos son los flujos que OBLIGATORIAMENTE deben testearse end-to-end:

- **Ventas:** Presupuesto → Pedido de Venta → Factura → Cobro
- **Compras:** Solicitud de Presupuesto → Pedido de Compra → Recepción → Factura Proveedor → Pago
- **Inventario:** Movimientos internos, transferencias entre almacenes, reglas de reabastecimiento
- **Fabricación (si aplica):** Lista de materiales → Orden de fabricación → Producción → Entrega
- **Contabilidad:** Asientos manuales, conciliación bancaria, cierre mensual/anual
- **RRHH/Nóminas (si aplica):** Alta empleado, cálculo de nómina, finiquitos
- **Integraciones externas:** APIs de terceros, importaciones automáticas, exportaciones de datos

---

## 2. Checklist Funcional Obligatorio (Fase 4.2)

| # | Flujo de negocio | Testeado por | Fecha | OK/KO | Incidencia (si KO) |
|---|---|---|---|---|---|
| 1 | Venta completa (presupuesto → pedido → factura → cobro) | | | | |
| 2 | Compra completa (solicitud → pedido → recepción → factura → pago) | | | | |
| 3 | Movimiento de inventario completo (transferencia + reabastecimiento) | | | | |
| 4 | Fabricación (si aplica): LdM → Orden → Producción → Entrega | | | | |
| 5 | Asiento manual + Conciliación bancaria | | | | |
| 6 | Nómina (si aplica): cálculo y generación | | | | |
| 7 | Integraciones externas (API, importaciones, exportaciones) | | | | |
| 8 | Informes clave del cliente (PDF, Excel, etc.) | | | | |
| 9 | Flujos adicionales específicos del cliente (especificar) | | | | |

---

## 3. Checklist Técnico Obligatorio (Fase 4.3)

> Las comprobaciones de infraestructura (versión PostgreSQL, restauración de backup, espacio en disco) NO van aquí: se validan en la Ficha de Infraestructura de Fase 1 y en el Checklist de Validación de Fase 5 (sección 4).

| # | Verificación técnica | Responsable | OK/KO |
|---|---|---|---|
| 1 | Módulos instalados, logs sin errores/warnings críticos y rendimiento aceptable (< 3s en vistas principales) | Tech | |
| 2 | Vistas principales y componentes JavaScript/OWL cargan sin errores | Tech | |
| 3 | Cron jobs (acciones planificadas) ejecutan correctamente | Tech | |
| 4 | Informes/reportes generan PDF correctamente | Tech | |
| 5 | Integraciones externas responden correctamente (APIs, webhooks) | Tech | |
| 6 | Emails/notificaciones envían correctamente | BC | |
| 7 | Permisos y grupos de seguridad correctos (access rights) | BC | |

---

## 4. Checklist de Infraestructura (Fase 5 — OBLIGATORIO, BLOQUEANTE)

> Origen: Post-mortem migración Akupalma (2026-02-22) — fallos por desajuste de versión PostgreSQL e insuficiencia de disco.
> NO se avanza a Fase 6 sin completar este checklist.

### Variante A: Servidores Propios (Cloud Indaws / On-premise)

#### A. PostgreSQL y Base de Datos

| # | Verificación | Comando/Método |
|---|---|---|
| A1 | Versión PostgreSQL destino >= versión del dump | `psql --version` en ambos entornos |
| A2 | Dump del ensayo restaurado exitosamente | `pg_restore` sin errores críticos |
| A3 | Odoo arranca sin errores críticos tras restauración | Log Odoo sin tracebacks bloqueantes |
| A4 | Neutralización ejecutada (crons, mail, IAP desactivados) | Verificar ir.cron + mail.server |
| A5 | Login exitoso en interfaz web | Acceder con usuario admin |

#### B. Almacenamiento y Disco

| # | Verificación | Comando/Método |
|---|---|---|
| B1 | Espacio libre >= 4x backup comprimido ANTES de restaurar | `df -h /` |
| B2 | Espacio libre >= 20% disco total DESPUÉS de restaurar | `df -h /` |
| B3 | Filestore copiado correctamente (assets e imágenes cargan) | Verificar en interfaz web |

#### C. Red y Conectividad

| # | Verificación | Comando/Método |
|---|---|---|
| C1 | Acceso SSH funcional al servidor destino | `ssh user@host` |
| C2 | Puertos abiertos (8069, 8072, 443) | `nc -zv host port` o `curl` |
| C3 | Acceso al bucket S3/almacenamiento de backups | Test descarga/subida de fichero |
| C4 | DNS configurado para dominio de producción | `dig domain.com` o `nslookup` |

#### D. Servidor y Servicios

| # | Verificación | Comando/Método |
|---|---|---|
| D1 | RAM suficiente (mínimo 4GB disponibles) | `free -h` |
| D2 | Servicio Odoo configurado y arrancado | `systemctl status odoo` |
| D3 | Reverse proxy con SSL configurado | `curl -I https://domain.com` |

### Variante B: Odoo.sh

| # | Verificación | Método |
|---|---|---|
| S1 | Versión PostgreSQL staging confirmada con soporte Odoo.sh | Ticket soporte / Panel Odoo.sh |
| S2 | Plan Odoo.sh soporta tamaño de BD | Verificar límites del plan contratado |
| S3 | Backup restaurado en rama staging sin errores | Panel Odoo.sh → Backups → Restaurar |
| S4 | Odoo arranca sin errores en staging | Logs en panel Odoo.sh |
| S5 | Filestore cargado correctamente | Verificar assets e imágenes en staging |
| S6 | Dominio de producción configurado en panel | Settings → Custom domains |
| S7 | Login exitoso en staging | Acceder con usuario admin |

### Firmas de Validación (obligatorias para ambas variantes)

| Rol | Nombre | Fecha | Firma |
|---|---|---|---|
| Developer | | | |
| Tech Lead | | | |

---

## 5. Criterios Go/No-Go (Fase 4.4)

### GO — Avanzar a Producción
- ✅ 100% flujos críticos funcionando (OK)
- ✅ 100% checklist técnico completado (OK)
- ✅ Cliente firma acta de conformidad

### NO-GO — Parar Proceso
- ❌ Cualquier flujo crítico KO → Parar, corregir, re-testear completamente
- ❌ Más del 10% del checklist técnico KO
- ❌ Cliente no firma o reporta incidencias bloqueantes

---

## 6. Prerrequisitos Obligatorios antes del Go-Live (Fase 6)

Verificación final antes de ejecutar producción:

- [ ] Checklist funcional FASE 4.2: Todos los flujos OK
- [ ] Checklist técnico FASE 4.3: Los 7 ítems OK
- [ ] Checklist infraestructura FASE 5: Todos los ítems OK + firmado por Dev y Tech Lead
- [ ] Ensayo FASE 5: Tiempos medidos y dentro de ventana aceptable
