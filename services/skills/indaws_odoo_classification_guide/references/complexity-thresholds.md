# Umbrales de Complejidad para Desarrollo Custom

Cuando la clasificacion es 🔴 Custom, usar estos umbrales para determinar el nivel de complejidad. Estos umbrales estan alineados con la clasificacion de complejidad de la skill `odoo-effort-estimator`; delega en ella para la fuente de verdad de la equivalencia complejidad↔horas.

## Zonas de Complejidad

### Desarrollo Menor (1-2 dias)

| Criterio | Umbral |
|----------|--------|
| Campos nuevos | ≤ 15 |
| Modelos nuevos | 0-1 simple |
| Lineas de Python | < 200 |
| Logica | Compute fields, onchange, constraints simples |
| Vistas | 1-2 vistas nuevas o modificaciones |
| Seguridad | Basica (1 grupo, access rights directos) |
| JS/OWL | No |
| APIs externas | No |

**Ejemplos:** campo computado no trivial, extension de wizard, constraint de negocio, modificar comportamiento de boton.

### Desarrollo Estandar (2-5 dias)

| Criterio | Umbral |
|----------|--------|
| Campos nuevos | 15-40 |
| Modelos nuevos | 1-3 |
| Lineas de Python | 200-800 |
| Logica | State machine, workflows, calculos complejos |
| Vistas | Multiples vistas con actions y menus |
| Seguridad | Media (2-3 grupos, record rules) |
| JS/OWL | Widget simple |
| APIs externas | No o integracion simple |

**Ejemplos:** gestion de proceso custom, sistema de aprobaciones, extension significativa de flujo, reporte PDF complejo, wizard multi-paso.

### Desarrollo Complejo (>5 dias)

| Criterio | Umbral |
|----------|--------|
| Campos nuevos | > 40 |
| Modelos nuevos | > 3 con relaciones complejas |
| Lineas de Python | > 800 |
| Logica | Integraciones, calculos avanzados, scheduling |
| Vistas | Vistas custom, portales, dashboards |
| Seguridad | Compleja (multiples grupos, reglas dinamicas) |
| JS/OWL | Componentes custom |
| APIs externas | Integraciones bidireccionales |

**Ejemplos:** integracion completa con sistema externo, portal personalizado, sistema de planificacion, modulo vertical completo, configurador frontend interactivo.

## Estimacion de Horas por Componente

Delega en la skill `odoo-effort-estimator`: es la fuente unica de horas por componente. No duplicar aqui.

## Clasificacion por LOC (para migraciones)

Horas por modulo segun LOC: delega en la skill `odoo-effort-estimator`. El
`INDAWS_ESTIMATION_MATRIX.csv` al que apuntaba esta seccion ya no existe; su contenido vive
ahora alli como matriz por componente.


## Ajustes por Calidad de Codigo (para migraciones)

Los ajustes por calidad de codigo (APIs deprecated, seguridad, rendimiento, OWL, mala calidad)
viven en la skill **`odoo-effort-estimator`**. Fuente unica — no duplicar los porcentajes aqui.
