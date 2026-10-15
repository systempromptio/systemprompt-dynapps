---
name: indaws-demo-customer-sentiment
description: >
  Analiza los tickets de helpdesk de los últimos 90 días agrupados por cliente,
  aplica análisis básico de sentimiento sobre el texto (título + descripción +
  chatter), evalúa la tendencia del volumen (0-30d vs 30-60d vs 60-90d) y
  cruza con las suscripciones activas (sale.order con subscription_state =
  3_progress). Devuelve un Churn Score 0-100 por cliente con evidencias,
  diagnóstico y acción recomendada. Detecta dos perfiles de riesgo opuestos:
  (1) cliente enfadado con muchos tickets negativos, (2) cliente desenganchado
  con suscripción activa y sin tickets. Usar cuando el usuario pida: clientes
  en riesgo, churn, cuentas en peligro, análisis de sentimiento, revisión de
  cartera de clientes.
version: 1.0.2
model: sonnet
effort: medium
---

# Demo · Customer sentiment

Radar de churn basado únicamente en tickets + suscripciones.

## Prerequisitos

- **odoo-pilot** (`helpdesk.ticket`, `mail.message`, `sale.order`, `res.partner`)
- Credenciales Odoo en env
- Python 3.8+ stdlib

## Workflow

| Paso | Accion | Output |
|------|--------|--------|
| 1 | Autenticar. Leer `config/config.json`. | -- |
| 2 | Ejecutar `python scripts/main.py` (default: 90 días). El script extrae tickets + sus chatter bodies + suscripciones activas, agrupa por cliente, calcula los 4 componentes del Churn Score, escribe `analysis.json` y HTML con placeholders. | analysis.json + draft.html |
| 3 | Leer `analysis.json`. Para **cada cliente con score ≥ 40** (ámbar o rojo), redactar: (a) **diagnóstico** (2-3 frases interpretando el por qué específico del riesgo), (b) **acción recomendada** (1-2 items del catálogo + responsable + plazo). Guardar en `output/narrative.json` como `{"per_customer": [{"partner_id": N, "diagnosis": "...", "actions": ["..."]}], "executive_summary": "..."}`. | narrative.json |
| 4 | Ejecutar `python scripts/main.py --finalize`. Aplica narrative al HTML → report final. | HTML final |

## Churn Score (0-100, más alto = más riesgo)

**1. Ticket health (30%)** · volumen tickets 90d · % críticos · % cancelados · tiempo resolución
**2. Sentiment (25%)** · léxico negativo vs positivo en title + description + chatter
**3. Trend (25%)** · compara bucket 0-30d vs 30-60d vs 60-90d — tanto **subida brusca** como **bajada brusca** penalizan
**4. Silence with subscription (20%)** · cliente con suscripción activa + volumen de tickets muy bajo o nulo en últimos 30d

Semáforo: 🔴 ≥70 · 🟡 40-69 · 🟢 <40

## Dos perfiles de riesgo

La skill detecta ambos:

### Perfil A · "Cliente enfadado"
Muchos tickets (algunos críticos, recurrentes) con lenguaje negativo → score alto por `ticket_health` + `sentiment`.

### Perfil B · "Cliente desenganchado"
Suscripción activa + pocos/ningún ticket reciente (drop >70% vs trimestre anterior) → score alto por `silence_with_subscription` + `trend`.

## Diagnóstico y acciones (paso 3)

### `diagnosis` por cliente (2-3 frases)
Interpreta el caso concreto. Ejemplos ilustrativos (varía estructura y énfasis según los datos):
> *"Bodegas Alcántara ha pasado de 4 tickets en Feb-Mar a 11 en abril, con 3 marcados como [RECURRENTE] y lenguaje repetidamente negativo en el chatter (menciones a 'otra vez', 'no resuelto'). La suscripción de mantenimiento sigue activa pero la confianza se erosiona. Perfil de cliente enfadado a punto de escalar."*
> *"Grupo Levante no abre tickets desde febrero y mantiene activa su suscripción de mantenimiento. No es satisfacción: el último ticket quedó sin respuesta del cliente. Perfil desenganchado con riesgo de no renovar."*

### `actions` (1-2 items del catálogo)
Formato: `"<acción del catálogo> · <responsable> · <plazo>"`. Ejemplo:
- `"Llamada urgente del Account Manager · Carla Ortiz · en las próximas 48h"`
- `"Auditoría técnica del sistema · Rubén Ferrer · esta semana"`

## RULES

1. **SIEMPRE** analizar texto normalizado (minúsculas, sin tildes). El léxico del config usa substrings (`"paralizad"` matchea "paralizada", "paralizados").
2. **NUNCA** inflar el score por sentiment sin evidencia — si un cliente tiene 1 ticket y 1 mención negativa, sentiment cuenta poco (hace falta masa)
3. El `ticket_count == 0` en últimos 30d + suscripción activa = **red flag por silencio**
4. Si el chatter tiene mensajes del propio equipo (Lumina), **también cuentan para el sentiment** — revelan cómo percibimos nosotros la relación
5. Priority mapping: `"3"` = Crítico · `"2"` = Premium · `"1"` = Prioritario · `"0"` = Normal

## GOTCHAS

1. `mail.message.body` es HTML — hay que limpiar tags antes del análisis léxico
2. Mensajes de tipo `notification` (creación automática) no aportan al sentiment — filtrar por `message_type in ('comment', 'email')`
3. `sale.order.subscription_state == '3_progress'` es la suscripción **activa** (valor estándar Odoo 19). Otros: `1_draft`, `2_renewal`, `5_renewed`, `6_churn`, `7_upsell`
4. Tickets cancelados (stage 5) **sí cuentan** para el histórico y el trend — cancelar es señal
5. Si un cliente tiene `ticket_count_total_90d < 2`, **ignorarlo** en sentiment (no hay señal suficiente)

## Delegates to
- **common:odoo-pilot**

## References
| File | Content |
|---|---|
| `config/config.json` | Pesos, léxico, umbrales, catálogo de acciones |
