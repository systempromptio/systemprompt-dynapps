# Estructura de Tareas de Migración

> Fuente operativa: `scripts/migration_spec.json` (las crea el motor `create_migration_tasks.py`). Sin subtareas.
> **Carril interno (18 tareas):** 8 core (`[MIG]` Análisis, Módulos [1:1/Core/Studio], BD, Testing, Go-Live, Soporte + `[INFRA]` Despliegue, Validación) + 8 `[GATE]` (0-7) + 2 gestión (Gestión del proyecto, Reuniones).
> **Carril Upgrade Lab (8 tareas):** 4 `[MIG]` + 3 `[GATE]` (A/B/C) + 1 gestión. Ver `upgrade-lab.md`.
> Abajo, las plantillas HTML de referencia de las tareas de trabajo principales.

---

## Arquitectura de tareas

```
Proyecto: Migración {Cliente} V{destino}    (project.project)
  ├── [INFRA] Despliegue de servidor         (ficha de infraestructura)
  ├── [MIG] Migración de módulos             (checklist de módulos)
  ├── [MIG] Migración de base de datos       (sin checklist)
  ├── [MIG] Testing funcional                (checklist de flujos)
  ├── [INFRA] Validación de infraestructura  (checks pre go-live)
  ├── [GATE] Gate 0 — Alcance y Propuesta
  ├── ...
  └── Gestión del proyecto, Reuniones, etc.
```

**Principio:** el proyecto de Odoo es el contenedor. No se crean tareas padre innecesarias. Cada tarea corresponde a una fase del proceso y se ve de un vistazo.

---

## Convención de Nombres y Tags

### Formato de nombre

| Tipo de tarea | Formato | Ejemplo |
|---|---|---|
| Despliegue de servidor | `[INFRA] Despliegue de servidor` | Fase 1 |
| Migración de módulos | `[MIG] Migración de módulos` | Fase 2 |
| Migración de BD | `[MIG] Migración de base de datos` | Fase 3 |
| Testing funcional | `[MIG] Testing funcional` | Fase 4 |
| Validación de infraestructura | `[INFRA] Validación de infraestructura` | Fase 5 |
| Tarea de gate | `[GATE] Gate {N} — {nombre}` | `[GATE] Gate 2 — Código Migrado` |
| Tarea de gestión | Sin prefijo `[MIG]` | `Reuniones`, `Gestión del proyecto` |

### Tags obligatorios (project.tags)

| Tag | Uso |
|---|---|
| **Migration** | Todas las tareas `[MIG]` y `[INFRA]` (marca genérica) |

Los tags de estrategia (Standard, Studio, Core, Custom) ya no se asignan a tareas individuales — la estrategia de cada módulo se indica en el checklist de la tarea de módulos.

---

## Tarea: [INFRA] Despliegue de servidor

**Fase:** 1 (Preparación técnica)
**Tag:** Migration
### Description HTML

```html
<div class="migration-task">
  <h3>Ficha de infraestructura del servidor destino</h3>
  <p>Completar ANTES de Gate 1. Detectar problemas de infra antes de invertir horas de desarrollo.</p>

  <h3>Datos del servidor</h3>
  <table class="table table-sm table-bordered">
    <thead>
      <tr><th>#</th><th>Dato</th><th>Valor</th><th>Notas</th></tr>
    </thead>
    <tbody>
      <tr><td>1</td><td>Tipo hosting destino</td><td></td><td>Cloud Indaws / Odoo.sh / Otro</td></tr>
      <tr><td>2</td><td>SO del servidor</td><td></td><td>ej. Ubuntu 22.04 LTS</td></tr>
      <tr><td>3</td><td>Versión PostgreSQL destino</td><td></td><td>psql --version</td></tr>
      <tr><td>4</td><td>Disco disponible</td><td></td><td>df -h /</td></tr>
      <tr><td>5</td><td>RAM</td><td></td><td>free -h</td></tr>
      <tr><td>6</td><td>CPU</td><td></td><td>nproc</td></tr>
      <tr><td>7</td><td>Versión PostgreSQL origen</td><td></td><td>Versión del dump</td></tr>
      <tr><td>8</td><td>Tamaño backup comprimido</td><td></td><td>Fichero .zip</td></tr>
      <tr><td>9</td><td>Tamaño BD descomprimida</td><td></td><td>dump.sql</td></tr>
      <tr><td>10</td><td>Tamaño filestore</td><td></td><td>Directorio filestore</td></tr>
    </tbody>
  </table>

  <h3>Checklist de conectividad</h3>
  <ul class="o_checklist">
    <li>Acceso SSH confirmado (usuario + clave)</li>
    <li>Acceso S3/bucket para backups</li>
    <li>Puertos abiertos (8069, 8072, 443)</li>
    <li>DNS/dominio configurado</li>
    <li>VPN configurada (si aplica)</li>
    <li>PostgreSQL destino >= versión del dump</li>
    <li>Espacio libre >= 4x backup comprimido</li>
  </ul>

  <h3>Log de avance</h3>
  <p><em>(Más reciente primero)</em></p>
  <ul>
    <li><strong>[Pendiente]</strong> Tarea creada automáticamente.</li>
  </ul>
</div>
```

### Uso
- El Dev/Sysadmin rellena cada campo del checklist al provisionar el servidor.
- Se completa ANTES del Gate 1.
- El TL revisa que todos los datos estén correctos antes de aprobar Gate 1.
- Detecta problemas de infraestructura antes de invertir horas de desarrollo.

---

## Tarea 1: [MIG] Migración de módulos

**Fase:** 2 (Migración de código)
**Tag:** Migration
**Planned hours:** Suma de horas de todos los módulos del análisis

### Description HTML

```html
<div class="migration-task">
  <h3>📋 Resumen</h3>
  <p>{n_modulos} módulos | {total_horas}h estimadas | v{origen} → v{destino}</p>

  <h3>📊 Módulos</h3>
  <table class="table table-sm table-bordered">
    <thead>
      <tr><th>Módulo</th><th>Estrategia</th><th>Horas</th><th>Riesgo</th><th>Dependencias</th></tr>
    </thead>
    <tbody>
      <tr><td>{module_name}</td><td>{strategy}</td><td>{hours}h</td><td>{risk}</td><td>{deps}</td></tr>
      <!-- ... un tr por módulo -->
    </tbody>
  </table>

  <h3>✅ Progreso</h3>
  <ul class="o_checklist">
    <li>{module_name} — {strategy} — {hours}h</li>
    <li>{module_name} — {strategy} — {hours}h</li>
    <!-- ... un li por módulo, checkbox nativo Odoo -->
  </ul>

  <h3>📝 Log de avance</h3>
  <p><em>(Más reciente primero — Dev actualiza tras cada sesión de trabajo)</em></p>
  <ul>
    <li><strong>[Pendiente]</strong> Tarea creada automáticamente desde análisis.</li>
  </ul>
</div>
```

### Uso
- El Dev marca cada módulo en el checklist cuando lo completa.
- La tabla resumen da visibilidad de todo el alcance de un vistazo.
- El TL/PM ve el progreso sin abrir subtareas.

---

## Tarea 2: [MIG] Migración de base de datos

**Fase:** 3 (Migración de BD)
**Tag:** Migration
**Planned hours:** Horas estimadas para el proceso de BD (parámetro `--db-hours`, obtenido del análisis)

### Description HTML

```html
<div class="migration-task">
  <h3>📋 Plan de migración de base de datos</h3>
  <p>Migración v{origen} → v{destino} | {tamaño} proyecto</p>

  <h3>🔄 Iteraciones</h3>
  <table class="table table-sm table-bordered">
    <thead>
      <tr><th>#</th><th>Iteración</th><th>Entorno</th><th>Resultado</th><th>Fecha</th></tr>
    </thead>
    <tbody>
      <tr><td>1</td><td>Test 1</td><td>Testing</td><td><em>Pendiente</em></td><td></td></tr>
      <tr><td>2</td><td>Ensayo (Rehearsal)</td><td>Pre-producción</td><td><em>Pendiente</em></td><td></td></tr>
      <tr><td>3</td><td>Producción</td><td>Producción</td><td><em>Pendiente</em></td><td></td></tr>
    </tbody>
  </table>

  <h3>📝 Validaciones por iteración</h3>
  <ul>
    <li>Conteo de registros antes/después por modelo crítico</li>
    <li>Validación de relaciones (Many2one, One2many, Many2many)</li>
    <li>Verificación de adjuntos y ficheros binarios</li>
    <li>Validación de saldos contables y conciliaciones</li>
  </ul>

  <h3>📝 Log de avance</h3>
  <p><em>(Más reciente primero)</em></p>
  <ul>
    <li><strong>[Pendiente]</strong> Tarea creada automáticamente.</li>
  </ul>
</div>
```

### Iteraciones por tamaño
- **Pequeño:** Test → Producción (2 iteraciones)
- **Mediano:** Test 1 → Ensayo → Producción (3 iteraciones)
- **Grande:** Test 1 → Test 2 → Ensayo → Producción (4 iteraciones)

### Uso
- No tiene checklist — la BD se migra entera.
- El Dev documenta el resultado de cada iteración en la tabla.
- Los scripts pre/post-migrate se documentan en el log de avance.

---

## Tarea 3: [MIG] Testing funcional

**Fase:** 4 (Testing funcional)
**Tag:** Migration
**Planned hours:** Horas estimadas para testing (parámetro `--testing-hours`, obtenido del análisis)

### Description HTML

```html
<div class="migration-task">
  <h3>📋 Testing funcional</h3>
  <p>Verificación de flujos críticos en entorno de pre-producción.</p>

  <h3>✅ Flujos a testear</h3>
  <ul class="o_checklist">
    <li>Venta simple (sale.order → account.move → account.payment)</li>
    <li>Venta con envío (sale.order → stock.picking → account.move)</li>
    <li>Compra simple (purchase.order → account.move → account.payment)</li>
    <li>Compra con recepción (purchase.order → stock.picking → account.move)</li>
    <li>Transferencia interna de inventario</li>
    <li>Asiento contable manual</li>
    <li>Conciliación bancaria</li>
    <!-- Flujos adicionales específicos del cliente -->
  </ul>

  <h3>📝 Log de avance</h3>
  <p><em>(Más reciente primero — BC/Dev documenta resultado de cada flujo)</em></p>
  <ul>
    <li><strong>[Pendiente]</strong> Tarea creada automáticamente.</li>
  </ul>
</div>
```

### Flujos por defecto
Se incluyen los flujos del catálogo estándar (`references/flow-testing.md`). El BC puede añadir flujos específicos del cliente al checklist.

### Uso
- El BC/Dev marca cada flujo como OK cuando lo verifica.
- Si un flujo es KO, se documenta en el log de avance con detalle.
- El Gate 4 no se aprueba hasta que todos los flujos estén OK.

---

## Tarea: [INFRA] Validación de infraestructura

**Fase:** 5 (Ensayo general / Rehearsal)
**Tag:** Migration
### Description HTML

```html
<div class="migration-task">
  <h3>Validación de infraestructura pre go-live</h3>
  <p>Completar durante Fase 5 (Rehearsal). BLOQUEANTE para Gate 5.</p>

  <h3>A. PostgreSQL y Base de Datos</h3>
  <ul class="o_checklist">
    <li>Versión PostgreSQL destino >= versión del dump</li>
    <li>Dump del ensayo restaurado exitosamente</li>
    <li>Odoo arranca sin errores críticos tras restauración</li>
    <li>Neutralización ejecutada (crons, mail, IAP desactivados)</li>
    <li>Login exitoso en interfaz web</li>
  </ul>

  <h3>B. Almacenamiento y Disco</h3>
  <ul class="o_checklist">
    <li>Espacio libre >= 4x backup comprimido ANTES de restaurar</li>
    <li>Espacio libre >= 20% disco total DESPUÉS de restaurar</li>
    <li>Filestore copiado correctamente (assets e imágenes cargan)</li>
  </ul>

  <h3>C. Red y Conectividad</h3>
  <ul class="o_checklist">
    <li>Acceso SSH funcional al servidor destino</li>
    <li>Puertos abiertos (8069, 8072, 443)</li>
    <li>Acceso al bucket S3/almacenamiento de backups</li>
    <li>DNS configurado para dominio de producción</li>
  </ul>

  <h3>D. Servidor y Servicios</h3>
  <ul class="o_checklist">
    <li>RAM suficiente (mínimo 4GB disponibles)</li>
    <li>Servicio Odoo configurado y arrancado</li>
    <li>Reverse proxy con SSL configurado</li>
  </ul>

  <h3>Tiempos medidos en rehearsal</h3>
  <table class="table table-sm table-bordered">
    <thead>
      <tr><th>Paso</th><th>Duración</th></tr>
    </thead>
    <tbody>
      <tr><td>Backup completo</td><td></td></tr>
      <tr><td>Restauración BD</td><td></td></tr>
      <tr><td>Restauración filestore</td><td></td></tr>
      <tr><td>Verificación</td><td></td></tr>
      <tr><td>Reapertura accesos</td><td></td></tr>
      <tr><td><strong>TOTAL</strong></td><td></td></tr>
    </tbody>
  </table>

  <h3>Firmas de validación</h3>
  <table class="table table-sm table-bordered">
    <thead>
      <tr><th>Rol</th><th>Nombre</th><th>Fecha</th><th>Firma</th></tr>
    </thead>
    <tbody>
      <tr><td>Developer</td><td></td><td></td><td></td></tr>
      <tr><td>Tech Lead</td><td></td><td></td><td></td></tr>
    </tbody>
  </table>

  <h3>Log de avance</h3>
  <p><em>(Más reciente primero)</em></p>
  <ul>
    <li><strong>[Pendiente]</strong> Tarea creada automáticamente.</li>
  </ul>
</div>
```

### Variantes
- **Variante A (Servidores propios):** Incluye checks de PostgreSQL, disco, red, servicios.
- **Variante B (Odoo.sh):** Incluye checks específicos de la plataforma.
El script detecta el tipo de hosting del servidor (indicado en la tarea [INFRA] Despliegue) para generar la variante correcta.

### Uso
- Se ejecuta durante el ensayo general (Fase 5).
- BLOQUEANTE: No se avanza a Fase 6 sin completar este checklist.
- Requiere firma de Dev + TL.

---

## Reglas de Uso

1. **Dev actualiza el log de avance** cada vez que trabaja en una tarea.
2. **Las decisiones de BC** se documentan siempre con fecha y motivo.
3. **TL puede leer cualquier tarea** y entender el estado sin preguntar.
4. **Sin subtareas:** todo el progreso se ve en el checklist de cada tarea.
5. **Sin subtareas:** una tarea por hito de fase; la lista exacta por carril y sus checklists viven en `scripts/migration_spec.json`.

---

## Nota sobre Horas

- **Migración de módulos:** `planned_hours` = suma de horas de todos los módulos del análisis.
- **Migración de BD:** `planned_hours` = valor del parámetro `--db-hours` (obtenido del análisis).
- **Testing funcional:** `planned_hours` = valor del parámetro `--testing-hours` (obtenido del análisis).
- **Despliegue de servidor** y **Validación de infraestructura:** No llevan horas planificadas (tareas de checklist operativo).

No existen valores por defecto. Las horas se obtienen del documento de análisis del proyecto.
