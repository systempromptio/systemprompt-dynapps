# Template: Resultado de Crear Tareas de Migración

> Usar tras ejecutar create_migration_tasks.py para mostrar el resultado al usuario.
> Herramienta de soporte: "Crea las tareas de migración"

---

## Tareas de Migración Creadas

**Proyecto:** {nombre_proyecto}
**Versión:** v{version_origen} → v{version_destino}
**Total horas estimadas:** {total_horas}h

### Tareas creadas

| # | ID | Tarea | Horas | Contenido |
|---|---|---|---|---|
| 1 | #{task1_id} | [INFRA] Despliegue de servidor | — | Ficha infra + checklist conectividad |
| 2 | #{task2_id} | [MIG] Migración de módulos | {horas_modulos}h | {n_modulos} módulos con checklist |
| 3 | #{task3_id} | [MIG] Migración de base de datos | {horas_bd}h | {n_iteraciones} iteraciones ({tamaño}) |
| 4 | #{task4_id} | [MIG] Testing funcional | {horas_testing}h | {n_flujos} flujos con checklist |
| 5 | #{task5_id} | [INFRA] Validación de infraestructura | — | Checklist pre go-live ({variante}) |

### Resumen por estrategia (módulos)
| Estrategia | Módulos | Horas | % del total |
|---|---|---|---|
| Standard | {n} | {h}h | {p}% |
| Studio | {n} | {h}h | {p}% |
| Core | {n} | {h}h | {p}% |
| Custom | {n} | {h}h | {p}% |

### Resumen por riesgo (módulos)
| Riesgo | Módulos | Horas |
|---|---|---|
| Bajo | {n} | {h}h |
| Medio | {n} | {h}h |
| Alto | {n} | {h}h |

### Enlaces directos
- [INFRA] Despliegue de servidor → {odoo_url}/odoo/project/{project_id}/tasks/{task1_id}
- [MIG] Migración de módulos → {odoo_url}/odoo/project/{project_id}/tasks/{task2_id}
- [MIG] Migración de base de datos → {odoo_url}/odoo/project/{project_id}/tasks/{task3_id}
- [MIG] Testing funcional → {odoo_url}/odoo/project/{project_id}/tasks/{task4_id}
- [INFRA] Validación de infraestructura → {odoo_url}/odoo/project/{project_id}/tasks/{task5_id}

### Siguiente paso
El Dev puede invocar **"¿Por dónde empiezo?"** para obtener el plan de trabajo ordenado por dependencias.

---

## Reglas de formato

- Mostrar SIEMPRE el ID de tarea y el enlace directo
- Incluir resúmenes por estrategia y riesgo de los módulos
- Si alguna tarea no se pudo crear (error API), listarla con el error
- Confirmar con el usuario ANTES de crear — mostrar preview (--dry-run) primero
