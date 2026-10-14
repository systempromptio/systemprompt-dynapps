# Horas por Componente Tecnico

Descomposicion de un desarrollo Odoo en componentes. **Unica tabla de horas por componente del
ecosistema.**

Son **horas base, sin factor IA**. El factor se aplica despues, por componente, segun
`ai-factors.md`.

**Alcance:** trabajo tecnico de desarrollo y migracion de version.

---

## Modelos y logica Python

| Componente | Horas |
|---|---|
| Extension de modelo (3-5 campos) | 1-2h |
| Campo computed con `depends` | 1-2h |
| Constraint SQL / Python | 1-2h |
| Override de metodo existente | 2-4h |
| Modelo nuevo simple (5-10 campos) | 2-4h |
| Modelo nuevo complejo (>15 campos, constraints) | 4-8h |
| Logica de negocio multi-paso | 4-8h por flujo |

## Vistas backend

| Componente | Horas |
|---|---|
| Anadir campo (xpath simple) | 0.5-1h |
| Pagina / tab en form | 1-2h |
| Vista form + list basica | 1-2h |
| Vista form compleja (pages, widgets) | 2-4h |
| Vista kanban custom | 2-3h |
| Conjunto completo (tree + search + form) | 4-6h |
| Dashboard / vista estadistica | 4-8h |

## Frontend OWL / JS

| Componente | Horas |
|---|---|
| Widget simple | 4-6h |
| Componente OWL con estado | 8-12h |
| Modificacion de ListRenderer / FormRenderer | 6-12h |
| Integracion de servicio externo en frontend | 12-20h |

## Integraciones

| Componente | Horas |
|---|---|
| API REST simple (1-2 endpoints, documentada) | 8-12h |
| API REST compleja (OAuth, paginacion) | 16-24h |
| API SOAP / legacy | 20-40h |
| Webhook receptor | 4-8h |
| Webhook emisor | 6-10h |
| Sincronizacion bidireccional | 24-40h |
| Importacion masiva con transformacion | 8-16h |

## Migracion de version

Portar un modulo a la version destino. El tramo lo da la clasificacion del modulo
(nº de ficheros y complejidad de la logica); si hay inventario del scanner, usarlo.

| Modulo a portar | Horas |
|---|---|
| `low` — 1-2 ficheros, sin logica | 3-6h |
| `medium` — 3-8 ficheros, logica simple | 4-16h |
| `high` — 8-15 ficheros, logica compleja | 12-24h |
| `very_high` — 15+ ficheros, JS/OWL, integraciones | 24-40h |

Los rangos abarcan dos fuentes: el criterio del equipo y la mediana real imputada antes de que
existiera el skill de migracion. Elegir dentro del rango segun el caso.

### Desglose fino, cuando hay inventario del scanner

El tramo de arriba es un atajo. Con el inventario por modulo se estima por componente, y el
tramo dentro de cada uno lo da el **LOC**:

| Componente | Complejidad | Horas | Unidad |
|---|---|---|---|
| Python (modelos) | Baja (<50 LOC) | 2h | por modulo |
| Python (modelos) | Media (50-200 LOC) | 4h (+1h por cada 50 LOC extra) | por modulo |
| Python (modelos) | Alta (>200 LOC) | 8h + revision manual | por modulo |
| Vista XML backend | Baja (xpath simple) | 0.5h | por vista |
| Vista XML backend | Alta (rediseno completo) | 2h | por vista |
| Informe QWeb | Media | 2.5h | por informe |
| Componente JS/OWL | Critico (v16+) | 8h minimo | por componente |
| Integracion API externa | Alta | 5h + revision manual | por modulo |
| Dashboard / KPIs | Media | 3h + revision manual | por modulo |
| Scaffold de modulo | Fijo | 1h | por modulo |

**Migracion de datos** (se estima aparte del porting):

| Tipo | Horas | Unidad |
|---|---|---|
| Mapeo simple de campos (1:1) | 1.5h | por modelo |
| Mapeo relacional (M2O/M2M) | 2.5h | por relacion |
| Transformacion compleja | 3h | por campo |
| Binarios / adjuntos | 8h | por modelo |

**Los dos niveles son coherentes**: sumando esta matriz para un modulo tipo de cada tramo salen
3.5h / 9h / 19h / 33h, que caen dentro de 3-6 / 4-16 / 12-24 / 24-40. Si al desglosar te sales
del tramo, revisa el inventario antes que la tabla.

Base de datos, iteraciones y salto de version **no se estiman aqui**: dependen del volumen de
la BD y del numero de iteraciones, no del numero de componentes. Sus formulas viven en las skills
`migration-assistant` y `odoo-database-analyzer`, y sus horas tambien son base: el factor de
`ai-factors.md` se aplica encima.

## Otros componentes

> **20-30% es esfuerzo, no cobertura.** Es el porcentaje de HORAS que cuestan los tests sobre las
> horas de desarrollo. La cobertura objetivo del codigo es del **90%** (ver `ai-factors.md`).

| Componente | Horas |
|---|---|
| Wizard | 2-4h |
| Reporte PDF (QWeb) | 3-6h |
| Seguridad (grupos + record rules) | 1-2h |
| Tests unitarios | 20-30% del esfuerzo de desarrollo |
| Tour test | 2-4h por flujo |

## Overhead tecnico

Presente en toda tarea. **No lleva factor IA.**

| Componente | Horas | Incluye | No incluye |
|---|---|---|---|
| Analisis tecnico | 1-4h | Leer codigo, verificar viabilidad, sondear una instancia | Reuniones, acordar alcance |
| Code review + iteracion | 1-2h | Revision por otro dev y correcciones | — |
| Deploy + validacion tecnica | 2-4h | Subir y comprobar en el entorno | Validacion funcional con cliente, formacion |

---

## Que NO entra en estas horas

Reuniones con cliente, formacion, validacion funcional, interrupciones y revisiones de trabajo
ajeno **no se estiman por tarea**: se descuentan de la capacidad semanal del desarrollador
(ver `team-velocities.md`). Meterlas aqui las contaria dos veces.
