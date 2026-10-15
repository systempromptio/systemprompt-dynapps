# Resumen — Proyecto de migracion configurado

**Proyecto**: {project_name} (#{project_id})
**Cliente**: {client_name}
**Migracion**: v{from_version} → v{to_version}
**Estrategia**: {strategy}
**Hosting**: {hosting_type}

---

## Calendario

| Hito | Fecha |
|---|---|
| Kick-off tecnico | {date_start} |
| Gate 0 (Alcance aprobado) | {date_start} |
| Fin Fase 1 (Analisis + Infra) | {fase_1_end} |
| Fin Fase 2 (Codigo migrado) | {fase_2_end} |
| Fin Fase 3 (BD migrada) | {fase_3_end} |
| Fin Fase 4 (Testing OK) | {fase_4_end} |
| Fin Fase 5 (Ensayo OK) | {fase_5_end} |
| **Go-Live** | **{date_golive}** |
| Fin soporte post | {date_support_end} |

**Duracion total**: {duration_weeks} semanas (incluyendo {post_golive_days} dias de soporte).

---

## Tareas creadas ({total_tasks} en total)

### Tareas de migracion ({n_mig_tasks})

| # | Tarea | Fase | Horas | Inicio | Fin |
|---|---|---|---|---|---|
{mig_tasks_rows}

### Tareas [GATE] ({n_gates})

| Gate | Nombre | Aprueba | Fecha |
|---|---|---|---|
{gate_tasks_rows}

### Tareas de gestion ({n_mgmt})

| Tarea | Horas | Rango |
|---|---|---|
| Gestion del proyecto | 0 (imputacion) | {date_start} → {date_support_end} |
| Reuniones | 0 (imputacion) | {date_start} → {date_support_end} |

---

## Resumen de horas

| Concepto | Horas |
|---|---|
| Modulos custom ({n_modulos} modulos) | {hours_modules}h |
| Analisis tecnico | {hours_analysis}h |
| Migracion BD | {hours_db}h |
| Testing funcional | {hours_testing}h |
| Despliegue infra | {hours_infra}h |
| Validacion infra y ensayo | {hours_validation}h |
| Go-Live | {hours_golive}h |
| Soporte post-migracion | {hours_support}h |
| **TOTAL** | **{total_hours}h** |

---

## Inventario de modulos

| Estrategia | Nº modulos | Horas |
|---|---|---|
{inventory_summary_rows}

---

## Knowledge

**Articulo raiz**: [{project_name}]({knowledge_url}) (ID {root_article_id})

Hijos creados:
- Analisis Tecnico
- Scripts BD
- Ensayo
- Go-Live
- Retrospectiva
- Interno (NO visible al cliente)

**Cliente compartido**: {client_name} con permiso "read" en el articulo raiz.

---

## Siguientes pasos

1. **Validar Gate 0 antes de empezar Fase 1**
   - Ir a la tarea `[GATE] Gate 0 — Alcance y propuesta`
   - Marcar entregables como completados
   - El COO firma la aprobacion (cambia stage a "Hecho")

2. **Asignar responsables a cada tarea**
   - Las tareas se han creado SIN asignar. El PM asigna manualmente:
     - Dev → tareas [MIG] de codigo y BD
     - BC → tarea Testing + Go-Live (co-responsable)
     - PM → tareas de gestion

3. **Rellenar la ficha de infraestructura**
   - Ir a la tarea `[INFRA] Despliegue de servidor`
   - Completar tabla del servidor destino (SO, PostgreSQL, disco, RAM, etc.)
   - Esto desbloquea Gate 1

4. **Para guia conversacional dia a dia**
   - Invocar el skill `migration-assistant` cuando tengas dudas sobre la fase actual, validacion de gates, planificacion tecnica o informes al cliente.

---

## Comandos utiles

**Abrir el proyecto en Odoo**:
```
{odoo_url}/odoo/project/{project_id}
```

**Filtrar solo tareas de migracion (sin gestion)**:
```
Filtro: Tags = Migration
```

**Ver el Knowledge del proyecto**:
```
{odoo_url}/odoo/knowledge/article/{root_article_id}
```
