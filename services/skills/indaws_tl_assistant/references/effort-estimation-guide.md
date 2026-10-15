# Guía de Estimación de Esfuerzo — TL Odoo v17-v19

## Método: Descomposición + Riesgo + Rango

No estimes "a ojo". Descompón, ajusta, comunica un rango.

## Niveles Oficiales Indaws

| Nivel | Descripción | Quién valora |
|-------|-------------|-------------|
| **Baja** | 1 cambio aislado (campo, computed sencillo, campo en vista/PDF) | PM directamente |
| **Media** | ≥4 cambios pequeños agrupados | PM directamente |
| **Alta** | Integraciones conocidas, cambios moderados flujo estándar | PM + TL |
| **Muy Alta** | Integraciones complejas, cambios drásticos | **SIEMPRE Co-CTOs** |

Escalar a Co-CTOs: Muy Alta, migraciones, integraciones nuevas, >40h estimadas.

## Paso 1: Descomponer en Componentes

Horas base por componente: delega en la skill `odoo-effort-estimator`. Fuente unica; no hay tabla
propia aqui.

Descomponer SIEMPRE, incluido el overhead tecnico (analisis, code review, deploy). Una tarea
estimada en bloque pierde precision y aplica mal el factor IA.

## Paso 2: Factores de Riesgo (multiplicadores)

| Factor | Mult | Cuándo |
|--------|------|--------|
| Módulo nuevo | x1.3 | Desde cero: manifest, seguridad, menús |
| Integración externa | x1.5-2.0 | API tercero, docs variable, sandbox inestable |
| JavaScript/OWL | x1.3-1.5 | Debugging difícil, menos tooling |
| Sin spec funcional clara | x1.5 | "El cliente quiere algo así como..." |
| Dev junior asignado | x1.3-1.5 | Curva aprendizaje + más review |
| Primera vez del dev en esta área | x1.3 | Senior pero nunca tocó contabilidad/stock |
| Impacto datos existentes | x1.3 | Migración datos, scripts, rollback |
| Multi-company | x1.2 | Record rules, company_dependent, testing cruzado |
| Requisito rendimiento | x1.3 | Volumen alto datos |
| Versión Odoo nueva | x1.2 | v19 recién salida, menos docs |

Aplicar POR COMPONENTE, no al total. Máximo acumulado razonable: x3.0 (si más, dividir tarea).

## Paso 3: Tres Escenarios

- **Optimista**: horas base sin factores (todo sale bien a la primera)
- **Esperado**: horas con factores (lo que comunicas)
- **Pesimista**: esperado × 1.5 (peor caso razonable)

Comunicar ESPERADO como principal, rango [optimista-pesimista] como contexto.

## Paso 4: Factor IA

Aplicar **después** de los factores de riesgo y **antes** de la contingencia.

**Factores: delega en la skill `odoo-effort-estimator`.** Es la fuente de verdad; no hay tabla
propia aquí.

## Paso 5: Cifra final

**Una sola cifra: el esfuerzo real, con factor IA aplicado.** Es la que va a la tarea y la que
se comunica.

Además del desarrollo, la valoración incluye:

| Concepto | Cálculo |
|----------|---------|
| Desarrollo | Escenario esperado del paso 3, con factor IA del paso 4 |
| Consultoría | 1-2h validación flujo con cliente |
| Buffer iteraciones | 10-20% si hay riesgo de ida y vuelta |
| **Total** | Suma de los anteriores |

Si comercialmente se decide cotizar otra cosa, es un margen explícito que decide el PM sobre
esta cifra. **Nunca se produce una versión "sin IA" de la estimación**: el esfuerzo real es el
que es, y tiene que quedar registrado.

Comunicar como **rango**, presentando el total como escenario esperado.

## Paso 6: Validación

- ¿Tiene sentido vs tareas similares anteriores?
- ¿He contado tests (20-30%), review, deploy?
- ¿Un senior lo haría en las horas optimistas?

## Capacidad Real

Dev a 40h/semana: meetings 3-5h, reviews otros 2-4h, interrupciones 2-4h, formación 1-2h.
**Desarrollo efectivo: ~28h/semana.** Usar esto para calcular fechas.

## Heurísticas Rápidas

| Tipo | Rango |
|------|-------|
| Bug fix simple | 1-3h |
| Bug fix complejo | 4-8h |
| Feature pequeña (campo+vista+lógica) | 4-8h |
| Feature media (wizard, reporte, flujo) | 8-20h |
| Feature grande (modelo completo con flujo) | 20-40h |
| Integración API externa | 16-40h |
| Módulo nuevo completo | 40-80h |
| Migración módulo entre versiones | 3-40h/módulo según tramo (base, sin factor IA) — tabla en `odoo-effort-estimator` |

## Estimación de migración de módulos

Horas base por módulo y **factor IA**: delega en la skill `odoo-effort-estimator`. No hay tabla
propia aquí — el factor de portar código entre versiones vive en la fuente de verdad, medido
sobre proyectos cerrados.

**Usar el resultado para `allocated_hours`** en la tarea de Odoo: es el esfuerzo real que el dev
consumirá.

## Anti-patrones

- **"Son 2 horitas"**: no cuenta análisis, tests, review → descomponer
- **Un solo número**: falsa precisión → siempre rango
- **No contar tests**: son 20-30% del esfuerzo → componente explícito
- **Estimar en días**: un "día" puede ser 4h o 7h → siempre horas
- **Fecha sin buffer**: fecha = estimación / 28h_sem × 1.2
