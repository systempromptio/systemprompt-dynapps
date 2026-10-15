# Calculo Dinamico del Timeline de Migracion

## Objetivo

Calcular las semanas de cada fase de migracion en base al analisis real de modulos del cliente, en lugar de usar valores fijos. Las horas son un calculo interno — el PDF solo muestra semanas por fase.

---

## Paso A: Estimacion de Horas por Modulo

Usar la accion asignada en el Paso 3 (clasificacion de modulos) y la complejidad para estimar horas:

### Modulos a MIGRAR (custom)

| Complejidad | Horas estimadas | Incluye |
|-------------|----------------|---------|
| Baja (<500 LOC) | 8h | Adaptacion codigo + QA basico |
| Media (500-2000 LOC) | 20h | Migracion codigo + refactor parcial + QA |
| Alta (>2000 LOC) | 40h | Migracion compleja + refactor + QA extensivo |

### Modulos a REEMPLAZAR

| Tipo de reemplazo | Horas estimadas | Incluye |
|-------------------|----------------|---------|
| Por Indaws Core | 3h | Instalacion + configuracion + validacion |
| Por Odoo Studio | 5h | Reconstruccion + configuracion + validacion |

### Modulos a EVALUAR (OCA)

| Complejidad | Horas estimadas | Incluye |
|-------------|----------------|---------|
| Cualquiera | 6h | Investigacion disponibilidad + decision + config/migracion basica |

### Modulos a ELIMINAR

| Tipo | Horas estimadas |
|------|----------------|
| Cualquiera | 0.5h |

### Personalizaciones Studio

| Elemento | Horas estimadas |
|----------|----------------|
| Campo custom (ir.model.fields manual) | 0.5h por campo |
| Vista modificada (Studio) | 1.5h por vista |

### Automatizaciones e Informes

| Elemento | Horas estimadas |
|----------|----------------|
| Accion automatizada (base.automation) | 1h por accion |
| Informe custom (ir.actions.report) | 2h por informe |

---

## Paso B: Tareas Genericas (siempre incluidas)

Estas tareas se anaden al total independientemente del cliente:

| Tarea | Formula |
|-------|---------|
| Configuraciones Generales (Setup) | 12h fijas |
| Reuniones semanales | 3h x numero_semanas_totales |
| Soporte post go-live | Segun bolsa total (ver tabla) |

### Soporte post go-live

| Bolsa total | Horas soporte |
|-------------|---------------|
| ≤100h | 18h |
| 101-300h | 24h |
| 301-700h | 32h |
| >700h | 48h |

---

## Paso C: Calculo de Semanas Totales

### Capacidad semanal

- Capacidad efectiva: **25h/semana** (equipo tipico: 1 consultor + desarrollador parcial)
- Para proyectos grandes (>500h): **35h/semana** (equipo reforzado)

### Formula

```
horas_modulos = SUM(horas por cada modulo segun tablas del Paso A)
horas_studio = (studio_fields * 0.5) + (studio_views * 1.5)
horas_auto = (automated_actions * 1) + (custom_reports * 2)
horas_genericas = 12 + soporte_post_golive

subtotal = horas_modulos + horas_studio + horas_auto + horas_genericas

# Añadir reuniones (formula iterativa simplificada)
semanas_estimadas = ceil(subtotal / capacidad_semanal)
horas_reuniones = 3 * semanas_estimadas
horas_totales = subtotal + horas_reuniones

# Recalcular semanas finales
semanas_totales = ceil(horas_totales / capacidad_semanal)
```

---

## Paso D: Distribucion en 3 Fases

### Porcentajes de distribucion

| Fase | Nombre | % del total | Minimo | Contenido |
|------|--------|-------------|--------|-----------|
| Fase 1 | Analisis y Migracion Tecnica | 50% | 4 semanas | Setup + Config modulos + Desarrollo custom |
| Fase 2 | Datos y UAT | 35% | 3 semanas | Migracion datos + Testing + Ajustes |
| Fase 3 | Go-Live y Soporte | 15% | 2 semanas | Formacion + Go-Live + Soporte post-arranque |

### Formula por fase

```
fase_1_semanas = max(ceil(semanas_totales * 0.50), 4)
fase_2_semanas = max(ceil(semanas_totales * 0.35), 3)
fase_3_semanas = max(2, ceil(semanas_totales * 0.15))

# Ajustar si la suma no coincide con el total
total_fases = fase_1_semanas + fase_2_semanas + fase_3_semanas
if total_fases < semanas_totales:
    fase_1_semanas += (semanas_totales - total_fases)  # absorbe la diferencia
elif total_fases > semanas_totales:
    semanas_totales = total_fases  # el minimo manda
```

### Regla: Fase 3 siempre fija en 2 semanas

La Fase 3 (Go-Live y Soporte) es operativa y no depende de la complejidad tecnica. Siempre son 2 semanas:
- Semana 1: Formacion final + Go-Live (fin de semana)
- Semana 2: Soporte intensivo post-arranque

Solo aumenta a 3 semanas si la bolsa total supera 500h (proyectos muy grandes).

---

## Ejemplos de Calculo

### Ejemplo 1: Cliente sencillo
- 2 modulos MIGRAR (Baja): 2 * 8h = 16h
- 1 modulo REEMPLAZAR (Core): 3h
- 2 modulos EVALUAR: 2 * 6h = 12h
- 3 modulos ELIMINAR: 3 * 0.5h = 1.5h
- 10 campos Studio: 10 * 0.5h = 5h
- 5 vistas Studio: 5 * 1.5h = 7.5h
- Genericas: 12h + 18h = 30h
- Subtotal: 75h
- Semanas est.: ceil(75/25) = 3 → Reuniones: 9h → Total: 84h
- **Semanas totales: ceil(84/25) = 4** (pero minimos aplican)
- Fase 1: max(ceil(4*0.50), 4) = 4 semanas
- Fase 2: max(ceil(4*0.35), 3) = 3 semanas
- Fase 3: 2 semanas
- **Total: 9 semanas**

### Ejemplo 2: Cliente complejo
- 13 modulos MIGRAR: 3 Alta (120h) + 5 Media (100h) + 5 Baja (40h) = 260h
- 3 modulos REEMPLAZAR (Core): 9h
- 3 modulos EVALUAR: 18h
- 10 modulos ELIMINAR: 5h
- 12 campos Studio: 6h
- 10 vistas Studio: 15h
- 1 accion automatizada: 1h
- 56 informes custom: 112h
- Genericas: 12h + 32h = 44h
- Subtotal: 470h
- Semanas est.: ceil(470/25) = 19 → Reuniones: 57h → Total: 527h
- **Semanas totales: ceil(527/35) = 16** (capacidad reforzada por >500h)
- Fase 1: max(ceil(16*0.50), 4) = 8 semanas
- Fase 2: max(ceil(16*0.35), 3) = 6 semanas
- Fase 3: 3 semanas (>500h)
- **Total: 17 semanas**

---

## Placeholders del Template

Los valores calculados se inyectan en estos placeholders del HTML:

- `{{phase_1_weeks}}` — Semanas de Fase 1
- `{{phase_2_weeks}}` — Semanas de Fase 2
- `{{phase_3_weeks}}` — Semanas de Fase 3
- `{{total_weeks}}` — Semanas totales
- `{{total_months}}` — Meses aproximados (ceil(semanas_totales / 4.33))

---

## Reglas Anti-Hallucination

- NUNCA inventar horas sin haber contado los modulos reales del Paso 3
- NUNCA mostrar horas en el PDF — solo semanas por fase
- SIEMPRE respetar los minimos (4 + 3 + 2 = 9 semanas minimo)
- SIEMPRE usar capacidad reforzada (35h/semana) si subtotal > 500h
- Si no hay datos suficientes para calcular (ej: no se pudo clonar el repo), usar los valores por defecto: 8 + 6 + 2 = 16 semanas con nota "(estimacion orientativa)"
