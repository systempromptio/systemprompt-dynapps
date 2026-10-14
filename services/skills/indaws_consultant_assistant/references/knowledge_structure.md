# Estructura Knowledge

## Qué es

La estructura de artículos Knowledge que todo proyecto Odoo de Indaws debe tener. Define los artículos obligatorios, condicionales y opcionales bajo el artículo raíz del proyecto, y cuándo crear cada uno.

## Cuándo aplica

- Al crear el proyecto en Odoo (fase 3: pre-proyecto)
- Durante la ejecución para documentar configuraciones, análisis y formaciones
- En la auditoría de estándares (Estándar #3)

## Qué debe saber el consultor

### Jerarquía esperada

Bajo el artículo raíz del proyecto (vinculado via `x_studio_knowledge` en project.project):

```
📄 {Nombre del proyecto}
├── 📄 Kick-Off                    [OPCIONAL]
├── 📄 Análisis                    [OBLIGATORIO]
│   ├── 📄 {App 1}
│   ├── 📄 {App 2}
│   └── 📄 ...
├── 📄 Cuestionarios de Análisis   [OPCIONAL]
├── 📄 Actas de reunión            [OBLIGATORIO]
├── 📄 Desarrollos                 [CONDICIONAL]
├── 📄 Adaptaciones                [CONDICIONAL]
├── 📄 Plan de Formaciones         [CONDICIONAL]
└── 📄 Interno                     [OPCIONAL]
```

### Clasificación de artículos

| Artículo | Clasificación | Condición |
|----------|---------------|-----------|
| Kick-Off | Opcional | Se recomienda siempre |
| Análisis | **Obligatorio** | Siempre debe existir, con sub-artículos por app |
| Cuestionarios de Análisis | Opcional | Si se generaron cuestionarios con questionnaire-generator |
| Actas de reunión | **Obligatorio** | Siempre debe existir |
| Desarrollos | Condicional | Si existen tareas con categoría [DEV] en checklists |
| Adaptaciones | Condicional | Si existen items de tipo STUDIO/AUTO en checklists |
| Plan de Formaciones | Condicional | Si existe tarea de formación en el proyecto |
| Interno | Opcional | Para notas internas del equipo |

### Contenido esperado por artículo

#### Análisis
- Sub-artículo por cada app del proyecto
- Contenido: análisis funcional generado por `project-analysis`
- Los sub-artículos se crean cuando se genera el análisis
- Debe reflejar GAPs detectados, soluciones propuestas y decisiones

#### Actas de reunión
- Un registro por reunión relevante (kick-off, seguimiento, análisis, formación)
- Formato: fecha, asistentes, temas tratados, decisiones, acciones pendientes
- Se pueden generar con `project-meeting-minutes`

#### Desarrollos
- Documentación funcional de cada módulo custom
- Generado por `odoo-module-documenter`
- Incluye: descripción, modelos, vistas, flujos, dependencias

#### Adaptaciones
- Cambios realizados via Odoo Studio o configuraciones complejas
- Documentado por `odoo-configurator` (modo STUDIO/AUTO)
- Incluye: campo/acción creada, modelo afectado, motivo, impacto

#### Plan de Formaciones
- Calendario y materiales de formación
- Generado por `training-planner`
- Incluye: sesiones planificadas, perfiles de usuario, materiales por sesión

### Reglas de validación

Para verificar la estructura Knowledge de un proyecto:

1. **Verificar artículo raíz**: `x_studio_knowledge` en project.project != false
2. **Buscar hijos**: Buscar knowledge.article con parent_id = artículo raíz
3. **Verificar obligatorios**: Buscar por ilike los nombres "Análisis" y "Actas"
4. **Verificar condicionales**: Solo si existen las condiciones correspondientes (tareas DEV, items STUDIO, etc.)
5. **Contar hijos**: El artículo raíz debe tener al menos 2 hijos (los obligatorios)

### Lógica de detección

```python
# Buscar artículo raíz
root_article_id = project.x_studio_knowledge

# Buscar hijos
children = search('knowledge.article', [('parent_id', '=', root_article_id)])

# Verificar obligatorios
has_analisis = any(c for c in children if 'Análisis' in c.name or 'Analisis' in c.name)
has_actas = any(c for c in children if 'Actas' in c.name or 'actas' in c.name)

# Verificar condicionales
has_dev_tasks = bool(search('xx.checklist.item', [
    ('task_id.project_id', '=', project_id),
    ('type', 'in', ['development'])
]))
# Si has_dev_tasks → verificar artículo "Desarrollos"
```

## Qué debe hacer el consultor

1. Al crear el proyecto, asegurar que el artículo raíz existe y está vinculado
2. Crear los artículos obligatorios (Análisis + Actas) inmediatamente
3. Crear los condicionales cuando se activen sus condiciones
4. Mantener el contenido actualizado durante todo el proyecto
5. Verificar la estructura en cada auditoría de estándares

## Skills relacionados

- `project-odoo-creator`: Crea el artículo raíz y estructura inicial
- `project-analysis`: Genera contenido del artículo "Análisis"
- `odoo-configurator`: Documenta en "Adaptaciones"
- `odoo-module-documenter`: Documenta en "Desarrollos"
- `training-planner`: Genera contenido de "Plan de Formaciones"
- `project-meeting-minutes`: Genera contenido de "Actas de reunión"

## Errores comunes

1. **No vincular x_studio_knowledge**: El proyecto existe pero no tiene artículo raíz asociado
2. **Crear el artículo raíz sin hijos**: La estructura vacía no aporta valor
3. **No crear "Análisis" como obligatorio**: Es el artículo más importante del proyecto
4. **Olvidar los condicionales**: Si hay desarrollos pero no artículo "Desarrollos", la documentación se pierde
5. **Nombres inconsistentes**: Usar "analisis" en vez de "Análisis" dificulta la validación automática
