# Template: Guion Interno de Demo

## Instrucciones de Uso

Este template se rellena con los datos de la oportunidad y la seleccion de flujos.
El LLM reemplaza los placeholders `{...}` y genera N bloques de flujo segun la seleccion.

**Este documento es INTERNO para el consultor, NO se comparte con el cliente.**

---

## Template

```markdown
# Guion de Demo: {client_name}

**Oportunidad**: {opportunity_name}
**Consultor**: {consultant_name}
**Fecha demo**: {demo_date}
**Duracion**: {duration} minutos
**Formato**: {format_30_60_90}

---

## 1. Contexto del Cliente

**Empresa**: {client_name} ({industry})
**Contacto**: {partner_name}
**Ubicacion**: {city}, {country}

### Pain Points Detectados

{pain_points_list}

### Informacion de la Toma de Datos

{toma_de_datos_summary}

### Perfil

- **Sector**: {industry}
- **Ingresos esperados**: {expected_revenue}
- **Tags**: {tags}
- **Prioridad**: {priority}

---

## 2. Estructura de la Demo ({duration} min)

| Fase | Tiempo | Contenido |
|------|--------|-----------|
{timeline_rows}

---

## 3. Apertura ({opening_minutes} min)

### Preguntas de Engagement

{opening_questions}

### Frase de Apertura

> "Buenas {morning_afternoon}, {contact_first_name}. Gracias por dedicarnos este tiempo. Hoy vamos a ver como Odoo puede ayudarles con {main_pain_summary}. Antes de empezar, me gustaria entender un poco mejor como trabajan actualmente."

### Encuadre General (Before/After)

**HOY**: {current_situation_summary}

**CON ODOO**: {desired_outcome_summary}

---

## 4. Bloques de Demo

{demo_blocks}

---

## 5. Q&A ({qa_minutes} min)

### Preguntas Esperadas

{expected_questions}

### Respuestas Preparadas

{prepared_answers}

---

## 6. Cierre ({closing_minutes} min)

### Resumen de Valor

> "Hoy hemos visto como Odoo les permite {value_summary}. Los puntos clave son:
> {key_takeaways}

### Proximos Pasos Propuestos

{next_steps}

### Call-to-Action

> "{call_to_action}"

---

## 7. Notas Tecnicas (Solo Consultor)

### Estado de Datos Demo

{demo_data_status}

### Precauciones

{precautions_list}

### Plan B

- Si la conexion falla: Usar capturas de pantalla preparadas
- Si el cliente quiere ver algo no preparado: "Excelente pregunta, lo incluiremos en la siguiente sesion"
- Si el tiempo se acaba: Saltar directo al cierre con resumen de valor
```

---

## Formato de Bloque de Demo

Cada bloque sigue esta estructura (generar N bloques segun flujos seleccionados):

```markdown
### Bloque {n}: {flow_name} ({block_minutes} min)

**App**: {app_name}
**Pain que resuelve**: {pain_point}

#### Frame (Before/After)

> **ANTES**: "{before_description}"
> **DESPUES**: "{after_description}"

#### Pasos de Demo

| # | Accion | Que hacer | Talking Point |
|---|--------|-----------|---------------|
{flow_steps}

#### Momento WOW

**{wow_moment_name}**: {wow_description}

> Frase: "{wow_talking_point}"

#### Datos Necesarios

{required_data_checklist}

#### Precauciones

{block_precautions}

#### Transicion al Siguiente Bloque

> "{transition_phrase}"
```

---

## Reglas de Generacion

1. **Personalizar siempre**: Usar nombre del cliente, sector, pain points reales
2. **Talking points en segunda persona**: "Ustedes van a poder...", "Su equipo vera..."
3. **No mas bloques que el tiempo permita**: 30 min → 2, 60 min → 3-4, 90 min → 4-5
4. **Transiciones narrativas**: Cada bloque termina conectando con el siguiente
5. **Datos verificados**: Marcar con ✅ si el dato existe en demo, ⚠️ si hay que crearlo
6. **Pain points del campo `xx_data_collection`** tienen PRIORIDAD sobre inferencias del sector
7. **Si la descripcion esta vacia**: Basarse en tags, sector y toma de datos; AVISAR al consultor
