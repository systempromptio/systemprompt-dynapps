# Clasificacion de Complejidad (esfuerzo)

Umbrales de **esfuerzo (horas)** para desarrollos Odoo. **La decision Standard/Studio/Core/Custom y el criterio Studio-vs-desarrollo NO se resuelven aqui: los decide `odoo-classification-guide` (fuente de verdad unica).** Este fichero se usa DESPUES de clasificar, para estimar horas.

**Los umbrales de horas son base, sin asistencia IA.** Aplicar los factores de `ai-factors.md`.

---

## Antes de estimar

La alternativa (Estandar / Core / Studio / Custom) la decide **`odoo-classification-guide`**: escalera `Estandar > Core > Studio > Custom`, con el criterio de **codigo gobernado** (para logica no trivial, un modulo revisado/testeable/migrable es preferible a Studio; Studio solo para lo trivial y de bajo riesgo). Aqui se asume que la funcionalidad ya esta clasificada como desarrollo; solo estimamos el esfuerzo.

---

## Zonas de Complejidad

### STUDIO (configuracion visual, sin codigo)

| Criterio | Umbral |
|----------|--------|
| Campos nuevos | <= 20 |
| Modelos nuevos | 0 (solo extiende existentes) |
| Lineas Python | 0 (o < 50 en automatizaciones basicas, Odoo 18+) |
| Logica de negocio | Simple (condiciones, asignaciones, envio emails) |
| Vistas | Modificacion de existentes, no nuevas |
| Seguridad | Grupos estandar, sin record rules complejas |
| Frontend JS/OWL | No |
| APIs externas | No |
| Reportes PDF | No (o modificaciones menores) |
| Wizards | No |

**Esfuerzo**: Horas, no dias.

**Capacidades Studio por version:**

| Capacidad | 16.0 | 17.0 | 18.0 | 19.0 |
|-----------|------|------|------|------|
| Campos custom | Si | Si | Si | Si |
| Vistas custom | Si | Si | Si | Si |
| Automatizaciones | Basicas | Mejoradas | Con Python | Con Python |
| Campos computados | No | Formula basica | Formula basica | Formula mejorada |
| Approval flows | No | No | Basico | Mejorado |
| Reportes custom | Limitado | Limitado | Mejorado | Mejorado |

### MINOR (modulo simple, 1-2 dias)

| Criterio | Umbral |
|----------|--------|
| Campos nuevos | <= 15 |
| Modelos nuevos | 0-1 simple |
| Lineas Python | 50-200 |
| Logica | Media (compute, onchange, constraints simples) |
| Vistas | 1-2 nuevas o modificaciones sustanciales |
| Seguridad | Basica (1 grupo, access rights directos) |
| Frontend JS/OWL | No |
| APIs externas | No |

**Esfuerzo**: 8-16h

### STANDARD (modulo medio, 2-5 dias)

| Criterio | Umbral |
|----------|--------|
| Campos nuevos | 15-40 |
| Modelos nuevos | 1-3 |
| Lineas Python | 200-800 |
| Logica | Compleja (state machine, workflows, calculos) |
| Vistas | Multiples con actions y menus |
| Seguridad | Media (2-3 grupos, record rules) |
| Frontend JS/OWL | Minimo (widget simple) |
| APIs externas | No o integracion simple |

**Esfuerzo**: 16-40h

### COMPLEX (modulo grande, > 5 dias)

| Criterio | Umbral |
|----------|--------|
| Campos nuevos | > 40 |
| Modelos nuevos | > 3 con relaciones complejas |
| Lineas Python | > 800 |
| Logica | Muy compleja (integraciones, calculos avanzados) |
| Vistas | Custom, portales, dashboards |
| Seguridad | Compleja (multiples grupos, reglas dinamicas) |
| Frontend JS/OWL | Componentes custom |
| APIs externas | Integraciones bidireccionales |

**Esfuerzo**: 40-120h+

---

## Arbol de Decision Rapido (esfuerzo)

> La rama Estandar / Core / Studio / Custom la decide `odoo-classification-guide`. Este arbol solo dimensiona el **esfuerzo** cuando el veredicto ya es desarrollo.

```
Veredicto = desarrollo (via odoo-classification-guide)
|
+-- < 200 lineas, <= 1 modelo    --> [MINOR]  8-16h
|
+-- 200-800 lineas, 1-3 modelos  --> [STANDARD] 16-40h
|
+-- > 800 lineas, > 3 modelos    --> [COMPLEX] 40-120h+
```

---

## Casos Frontera

### Studio con Python basico (disponible desde 14.0, no desde 18.0)
- SI aplica: asignar valores, crear registros simples, enviar emails con condiciones
- NO aplica: herencia de modelos, override de metodos, compute fields con logica compleja
- El limite (trivial -> Studio; logica no trivial -> modulo gobernado) lo fija `odoo-classification-guide`, no el n. de lineas.

### Indaws Core + extension
- Si Core cubre 80% y el 20% restante es < 100 lineas → modulo bridge
- Si el 20% restante es complejo → evaluar crear desde cero
- Consultar equipo Core si el requisito faltante podria anadirse al modulo existente

### Migrabilidad como factor
- Un modulo bien escrito con tests migra MAS facil que un Studio complejo. Con IA, portar el modulo entre versiones es rapido: la vieja ventaja de "Studio migra solo" ya no pesa (ver `odoo-classification-guide`).
- Studio encaja para lo trivial y de bajo riesgo; para logica de negocio no trivial, desarrollo gobernado.
- Para versiones < 16.0, Studio muy limitado: preferir desarrollo.
