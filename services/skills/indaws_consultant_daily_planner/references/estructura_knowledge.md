# Estructura Knowledge del Proyecto

## Estructura Esperada

Bajo el articulo raiz (`x_studio_knowledge` de `project.project`), cada proyecto debe tener esta estructura de articulos hijos:

```
{Nombre del Proyecto}
├── Kick-Off
├── Análisis
│   ├── Análisis de Ventas
│   ├── Análisis de Compras
│   └── ... (1 por app del proyecto)
├── Cuestionarios de Análisis (opcional)
│   ├── Ventas
│   ├── Compras
│   └── ...
├── Actas de reunión
├── Desarrollos
│   ├── {Nombre del desarrollo 1}
│   └── {Nombre del desarrollo 2}
├── Adaptaciones
├── Plan de Formaciones
└── Interno
```

## Reglas de Validacion

### Articulos Obligatorios (siempre deben existir)

| Articulo | Condicion | Verificacion |
|----------|-----------|-------------|
| Analisis | Siempre | Debe existir Y tener al menos 1 hijo (articulo por app) |
| Actas de reunion | Siempre | Debe existir. Verificar que tiene contenido si ya hubo reuniones |

### Articulos Condicionales (segun el proyecto)

| Articulo | Condicion | Verificacion |
|----------|-----------|-------------|
| Desarrollos | Si hay tareas con tag [DEV] | 1 articulo hijo por cada tarea [DEV] |
| Adaptaciones | Si hay items STUDIO/AUTO en checklists | Debe existir |
| Plan de Formaciones | Si hay tarea de formaciones | Debe existir |

### Articulos Opcionales (no bloquean)

| Articulo | Proposito |
|----------|-----------|
| Kick-Off | Presentacion de kick-off del proyecto |
| Cuestionarios de Analisis | Cuestionarios previos. No es relevante para la agenda |
| Interno | Documentacion interna (health reports, guiones formacion, etc.) |

## Como Verificar

1. Leer `x_studio_knowledge` del proyecto → obtener ID del articulo raiz
2. Buscar articulos hijos: `knowledge.article` con `parent_id = root_id`
3. Comparar nombres de hijos con la lista esperada (busqueda por `ilike`, no exacta)
4. Para "Analisis": verificar que tiene sub-hijos (1 por app)
5. Para "Desarrollos": cruzar con tareas [DEV] del proyecto

## Acciones si Falta Algo

- Si falta un articulo obligatorio → incluir en agenda como IMPORTANTE
- Si falta "Desarrollos" y hay tareas [DEV] → incluir como PLANIFICADO
- Si falta "Actas de reunion" y ya hubo reuniones → incluir como URGENTE
- Nunca crear articulos automaticamente — solo avisar al consultor
