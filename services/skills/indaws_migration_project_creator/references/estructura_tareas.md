# Estructura de las 19 Tareas (carril interno)

Referencia del carril **interno**. Para Upgrade Lab ver `upgrade_lab.md`.

> Los checklists y entregables OPERATIVOS exactos (y sus conteos) son los de `migration-assistant/scripts/migration_spec.json` (fuente de verdad). Los conteos de gate de aquí abajo derivan de ese spec; si cambian, edítalos en el spec.

---

## Vision general

```
Proyecto: Migracion {Cliente} v{X}→v{Y}     (etapa "Migraciones")
│
├── Gestion del proyecto                    (sin prefijo, todo el proyecto)
├── Reuniones                               (sin prefijo, todo el proyecto)
│
├── [GATE] Gate 0 — Alcance y propuesta     ← aprueba COO
│
├── [INFRA] Despliegue de servidor          (Fase 1)
├── [MIG]   Analisis y clasificacion        (Fase 1)
├── [GATE]  Gate 1 — Analisis tecnico       ← aprueba Tech Lead
│
├── [MIG]   Migracion de modulos            (Fase 2 — segun estrategia 1:1/Core/Studio)
├── [GATE]  Gate 2 — Codigo migrado         ← aprueba Tech Lead
│
├── [MIG]   Migracion de base de datos      (Fase 3)
├── [GATE]  Gate 3 — BD migrada             ← aprueba Tech Lead
│
├── [MIG]   Testing funcional               (Fase 4)
├── [GATE]  Gate 4 — Testing funcional OK   ← aprueba Tech Lead
│
├── [INFRA] Validacion infra y ensayo       (Fase 5)
├── [GATE]  Gate 5 — Ensayo exitoso         ← aprueba Tech Lead
│
├── [MIG]   Puesta en produccion (Go-Live)  (Fase 6)
├── [GATE]  Gate 6 — Go-Live OK             ← aprueba TL + CTO Ops
│
├── [MIG]   Soporte post-migracion          (Fase 7)
└── [GATE]  Gate 7 — Cierre del proyecto    ← aprueba COO
```

**Total**: 18 tareas (8 mig + 8 gates + 2 gestion).

---

## Distribucion de fechas

El script reparte automaticamente entre `date_start` y `date_golive` segun:

| Fase | Peso (% del rango total) | Tarea | Observaciones |
|---|---|---|---|
| 0 | 1 dia fijo | Gate 0 | Al inicio (date_start) |
| 1 | 10% | Despliegue + Analisis | Tareas en paralelo |
| 2 | 30% | Modulos + Studio | Studio en 2ª mitad (50-100%) |
| 3 | 20% (25% si large) | Migracion BD | Iteraciones segun tamaño |
| 4 | 25% (20% si large) | Testing funcional | FASE CRITICA |
| 5 | 10% | Validacion infra + ensayo | BLOQUEANTE |
| 6 | 5% | Go-Live | Termina exactamente en date_golive |
| 7 | 14 dias fijos | Soporte | Despues de date_golive |

**Gate N**: 1 dia al final de su fase (validacion). Excepcion Gate 0: el dia date_start.

---

## Tareas de Migracion (8)

### 1. [MIG] Analisis y clasificacion de modulos — Fase 1

- **Sequence**: 100
- **Horas**: `--analysis-hours` (default 8h)
- **Tag**: Migration
- **Checklist**: SI — `migration_spec.json` → `checklists.analysis` + tabla resumen del inventario en HTML
- **Descripcion**: Estudio previo (`odoo-database-upgrader` FASE 1) + STEP 0 triaje (`odoo-module-upgrader`). Muestra resumen por estrategia.

### 2. [INFRA] Despliegue de servidor — Fase 1

- **Sequence**: 110
- **Horas**: `--infra-hours` (default 0)
- **Tag**: Migration
- **Checklist**: SI — `migration_spec.json` → `checklists.infra_deploy` + ficha de infra en HTML
- **Descripcion**: Tabla con campos del servidor a rellenar + checklist de acceso (SSH, S3, puertos, DNS).

### 3. [MIG] Migracion de modulos — Fase 2

- **Sequence**: 120
- **Horas**: suma de horas de modulos del inventario
- **Tag**: Migration
- **Checklist**: SI — 1 item por modulo (cada uno segun su estrategia: 1:1/Core/Studio) + `migration_spec.json` → `checklists.modules_fixed` (CI/CD verde, code review)
- **Descripcion**: Tabla con todos los modulos (nombre, estrategia, horas, riesgo, dependencias).

### 4. [MIG] Migracion de base de datos — Fase 3

- **Sequence**: 130
- **Horas**: `--db-hours`
- **Tag**: Migration
- **Checklist**: SI — `migration_spec.json` → `checklists.database` + tabla de iteraciones en HTML
- **Iteraciones**:
  - small: 2 (Test → Prod)
  - medium: 3 (Test → Ensayo → Prod)
  - large: 4 (Test 1 → Test 2 → Ensayo → Prod)

### 5. [MIG] Testing funcional — Fase 4

- **Sequence**: 140
- **Horas**: `--testing-hours`
- **Tag**: Migration
- **Checklist**: SI — `migration_spec.json` → `checklists.testing` + extras de `--extra-flows`
- **Tipo checklist**: TEST (peso 2)
- **FASE CRITICA**: principal causa de errores historicos.

### 6. [INFRA] Validacion de infraestructura y ensayo — Fase 5

- **Sequence**: 150
- **Horas**: `--validation-hours`
- **Tag**: Migration
- **Checklist**: SI — `migration_spec.json` → `checklists.infra_validation` + secciones HTML (PostgreSQL+BD, Almacenamiento, Red, Servidor, tiempos, firmas)
- **Variante Odoo.sh**: si `hosting=odoo-sh`, HTML simplificado sin checks de servidor propio.

### 7. [MIG] Puesta en produccion (Go-Live) — Fase 6

- **Sequence**: 160
- **Horas**: `--golive-hours` (default 8h)
- **Tag**: Migration
- **Checklist**: SI — `migration_spec.json` → `checklists.golive` + protocolo en HTML
- **Fechas**: termina EXACTAMENTE en date_golive.

### 8. [MIG] Soporte post-migracion y cierre — Fase 7

- **Sequence**: 170
- **Horas**: `--support-hours` (default 20h)
- **Tag**: Migration
- **Checklist**: SI — `migration_spec.json` → `checklists.support` + actividades en HTML
- **Fechas**: date_golive → date_golive + 14 dias.

---

## Tareas [GATE] (8)

Sequence: 1000, 1010, 1020, ..., 1070.

| Gate | Fase | Ejecuta | Aprueba | Items checklist |
|---|---|---|---|---|
| 0 | 0 | PM | COO | 7 |
| 1 | 1 | Dev | Tech Lead | 6 |
| 2 | 2 | Dev | Tech Lead | 5 |
| 3 | 3 | Dev | Tech Lead | 7 |
| 4 | 4 | BC + Dev | Tech Lead | 4 |
| 5 | 5 | Dev | Tech Lead | 5 |
| 6 | 6 | BC + Dev | Tech Lead + CTO Ops | 8 |
| 7 | 7 | PM + TL + BC | COO | 5 |

- **Tag**: Migration
- **Horas**: 0
- **Checklist**: SI — `migration_spec.json` → `gates.internal.{N}.checklist` (entregables, tipo ANLS, peso 1)
- **Duracion**: 1 dia al final de cada fase

Ejecutor/aprobador queda en el HTML para referencia. La aprobacion la registra el responsable cambiando el `stage_id` de la tarea a "Hecho" (manual).

---

## Tareas de Gestion (2)

Sequence: 2000, 2010.

| Tarea | Horas | Tag | Fechas |
|---|---|---|---|
| Gestion del proyecto | 0 (imputacion) | (ninguno) | date_start → date_golive + 14d |
| Reuniones | 0 (imputacion) | (ninguno) | date_start → date_golive + 14d |

**Importante**: NO llevan el tag `Migration`. Son tareas genericas de imputacion de horas que NO forman parte del ciclo de migracion.

Sin checklist. Solo descripcion explicando que es para imputar coordinacion/reuniones.

---

## Campos comunes a todas las tareas

```python
{
    'name': str,                       # Con prefijo [MIG]/[INFRA]/[GATE]
    'project_id': int,
    'allocated_hours': float,          # planned_hours en v18-
    'planned_date_begin': str,         # YYYY-MM-DD 08:00:00
    'date_deadline': str,              # YYYY-MM-DD 18:00:00
    'description': str,                # HTML con tablas, checklists, log
    'sequence': int,                   # Ver tabla arriba
    'user_ids': [[5]],                 # Vaciar (no auto-asignar)
    'tag_ids': [[4, migration_tag_id]],  # Excepto tareas de gestion
    'stage_id': 10,                    # "Sin asignar" (las gestiona Odoo despues)
}
```

---

## Validaciones post-creacion

Despues de invocar el script:

1. **Contar tareas**: `search_count` debe ser >= numero esperado (18 con todo, menos si flags off).
2. **Verificar fechas**: ninguna `date_deadline` antes de `planned_date_begin`.
3. **Verificar tag**: todas las [MIG]/[INFRA]/[GATE] llevan tag Migration.
4. **Verificar gestion sin tag**: "Gestion del proyecto" y "Reuniones" sin tag Migration.
5. **Verificar checklists**: si `--with-checklists`, count(xx.checklist.item WHERE task_id IN tasks) > 0.
