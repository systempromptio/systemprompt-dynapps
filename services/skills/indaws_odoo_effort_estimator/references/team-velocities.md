# Velocidades y Capacidades de Equipo

Datos base que determinan la duracion y distribucion de horas de cualquier proyecto Odoo.

Las velocidades son de capacidad, no de esfuerzo: **no llevan factor IA**. Reuniones, formacion e interrupciones ya estan descontadas aqui, no en las horas de cada tarea.

---

## Velocidades de Referencia

| Via | Capacidad semanal | Incluye | Responsable |
|-----|-------------------|---------|-------------|
| **Funcional** | 12h/semana | Configuracion, reuniones, formacion, migracion datos, soporte | Equipo consultoria |
| **Desarrollo** | 25h/semana | Desarrollo codigo, testing interno, revisiones, correcciones | Equipo desarrollo |
| **Gestion proyecto** | 1h/semana | Seguimiento, coordinacion, reporting | PM (fija, independiente) |

---

## Split Funcional 70/30

Las horas funcionales totales (sin gestion de proyecto ni desarrollos) se dividen:

```
horas_implantacion       = horas_funcionales × 0.70
horas_reuniones_formacion = horas_funcionales × 0.30
```

La migracion de datos se incluye dentro del 70% de implantacion (no es concepto separado).
Los formatos de impresion estandar se incluyen en implantacion; los custom son desarrollos.

---

## Dos Vias Paralelas

Todo proyecto tiene dos vias que avanzan en paralelo:

```
duracion_proyecto = max(semanas_funcional, semanas_desarrollo)
```

Si desarrollo es mas largo que funcional, se extiende la implantacion funcional con acompanamiento/soporte.

Los desarrollos SIEMPRE van en paralelo a la implantacion funcional.
Las apps funcionales se implantan de forma SECUENCIAL por defecto (se puede paralelizar si hay mas consultores).

---

## Soporte en Produccion

| Tipo proyecto | Semanas soporte | Horas |
|---------------|-----------------|-------|
| Estandar | 2 semanas | 24h |
| Complejo | 3-4 semanas | 36-48h |

Siempre incluido al final del proyecto.

---

## Calculo Completo

```
1. semanas_funcional = config_inicial(1) + sum(semanas_por_app) + soporte_produccion
2. horas_funcionales = semanas_funcional × 12h/semana
3. semanas_desarrollo = ceil(total_horas_desarrollo / 25)
4. duracion_proyecto = max(semanas_funcional, semanas_desarrollo)
5. horas_gestion = duracion_proyecto × 1h/semana
6. horas_implantacion = horas_funcionales × 0.70
7. horas_reuniones = horas_funcionales × 0.30
8. TOTAL = horas_funcionales + total_horas_desarrollo + horas_gestion
```

---

## Presentacion al Usuario

Siempre presentar:
1. Calendario por app (tabla semana × via funcional × via desarrollo)
2. Resumen de horas por via
3. Opciones de ajuste: paralelizar apps, extender duracion, reordenar prioridad
4. El usuario DEBE validar antes de continuar
