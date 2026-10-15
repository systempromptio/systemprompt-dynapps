# Template: Resumen de Creacion del Proyecto

Este template define la estructura del informe que se presenta al usuario tras completar la creacion.

---

## Formato del Resumen

```markdown
# Proyecto Configurado: {nombre_proyecto}

**Cliente**: {nombre_cliente}
**Proyecto Odoo**: [{nombre_proyecto}]({odoo_url}/odoo/project/{project_id})
**Knowledge**: [{nombre_proyecto}]({odoo_url}/knowledge/article/{root_article_id})

---

## Tareas Creadas ({total_tareas})

| # | ID | Tarea | Horas | Inicio | Fin | Checklist |
|---|-----|-------|-------|--------|-----|-----------|
| 1 | {id} | Configuraciones Generales | 12h | 01/04/2026 | 07/04/2026 | 8 items |
| 2 | {id} | CRM | 16h | 08/04/2026 | 18/04/2026 | 12 items |
| ... | ... | ... | ... | ... | ... | ... |

**Total**: {total_horas}h planificadas | {total_checklist_items} items de checklist

---

## Estructura Knowledge

| ID | Articulo | Icono | Compartido | Estado |
|----|----------|-------|------------|--------|
| {id} | {nombre_proyecto} (raiz) | 📁 | SI (cliente: {nombre_cliente}) | Creado |
| {id} | Kick-Off | 🚀 | SI | Creado |
| {id} | Analisis | 🔍 | SI | Creado |
| {id} | Cuestionarios de Analisis | 📋 | SI | Creado |
| {id} | Formaciones | 🎓 | SI | Creado |
| {id} | Actas de Reunion | 📝 | SI | Creado |
| {id} | Adaptaciones | ⚙️ | SI | Creado |
| {id} | Interno | 🔒 | NO (solo Indaws) | Creado |

**Total**: {total_articles} articulos creados

---

## Verificaciones

- [x] Proyecto existe y accesible: {nombre_proyecto} (ID: {project_id})
- [x] {total_tareas} tareas creadas correctamente
- [x] {total_checklist_items} items de checklist creados
- [x] Estructura Knowledge creada ({total_articles} articulos)
- [x] Campo x_studio_knowledge vinculado (article ID: {root_article_id})
- [x] Cliente {nombre_cliente} anadido como miembro del Knowledge (lectura)
- [x] Articulo "Interno" desincronizado (no visible para cliente)

---

## Proximos Pasos

El proyecto esta listo para operar. Los siguientes skills pueden ejecutarse sobre este proyecto:

1. **kick-off-presentation** → Genera presentacion y articulo Kick-Off
2. **questionnaire-generator** → Genera cuestionarios en Google Drive
3. **project-analysis** → Genera documentos de analisis por app
4. **project-plan** → (ya ejecutado, es el input de este skill)
5. **project-health-monitor** → Monitoriza la salud del proyecto
```

---

## Notas para el LLM

- Reemplazar todos los `{placeholders}` con datos reales
- Los links a Odoo usan el formato `{odoo_url}/odoo/project/{id}` para proyectos y `{odoo_url}/knowledge/article/{id}` para Knowledge
- La columna "Estado" puede ser "Creado" o "Ya existia" (si se encontro con search-before-create)
- Si alguna verificacion falla, marcarla con `[ ]` en lugar de `[x]` y explicar el problema
- Las fechas en el resumen se muestran en formato DD/MM/YYYY (formato humano), no en formato API
