# Competitive Intelligence - Workflow Detail

Reference file with detailed steps, battlecard template, and success criteria.

---

## Paso 0: Identificar Competidor

PREGUNTAR al usuario:
1. **Competidor** (nombre o tipo)
2. **Contexto** (deal concreto donde compiten, o battlecard general)
3. **Que necesita** (battlecard completa, o respuesta puntual a objecion)

> **DETENTE** y espera respuesta del usuario.

---

## Paso 1: Investigacion

### 1.1 Datos del CRM (via odoo-pilot)

Oportunidades ganadas contra este competidor:
```
Modelo: crm.lead
Dominio: [["stage_id.is_won","=",true],
          "|",["description","ilike","{competidor}"],
          ["tag_ids.name","ilike","{competidor}"]]
Campos: name, partner_name, expected_revenue, date_closed, description
```

Oportunidades perdidas contra este competidor:
```
Modelo: crm.lead
Dominio: [["active","=",false],["probability","=",0],
          "|",["lost_reason.name","ilike","{competidor}"],
          ["description","ilike","{competidor}"]]
Campos: name, partner_name, expected_revenue, lost_reason, description
```

### 1.2 Investigacion web (fuentes publicas)

- Web del competidor (producto, precios, clientes)
- Casos de exito publicados
- Reviews en G2, Capterra, Software Advice
- Noticias recientes (funding, adquisiciones, cambios)
- Presencia en redes sociales y eventos

### 1.3 Base de conocimiento (si existe)

- Consultar `competitor_landscape.md` para datos conocidos
- NotebookLM para investigacion profunda (si disponible)

---

## Paso 2: Generar Battlecard

PRESENTAR al usuario con este formato:

```
=============================================
  BATTLECARD: {competidor}
  Actualizada: {fecha}
=============================================

--- 1. OVERVIEW ---
Empresa:      {nombre}
Sede:         {ubicacion}
Tamano:       {empleados} empleados
Fundada:      {ano}
Mercado:      {mercado_objetivo}
Precio:       {modelo_precios_aproximado}
Partners:     {si_tiene_red_de_partners}
Web:          {url}

--- 2. PRODUCTO ---
Fortalezas:
  + {fortaleza_1}
  + {fortaleza_2}
  + {fortaleza_3}

Debilidades:
  - {debilidad_1}
  - {debilidad_2}
  - {debilidad_3}

Modulos/Areas clave: {areas_funcionales}

--- 3. DIFERENCIACION INDAWS + ODOO ---
| Area | {Competidor} | Indaws + Odoo | Ventaja |
|------|-------------|---------------|---------|
| {area_1} | {su_enfoque} | {nuestro_enfoque} | {quien_gana_y_por_que} |
| {area_2} | {su_enfoque} | {nuestro_enfoque} | {quien_gana_y_por_que} |
| {area_3} | {su_enfoque} | {nuestro_enfoque} | {quien_gana_y_por_que} |

Nuestros diferenciadores clave:
1. {diferenciador_1_con_evidencia}
2. {diferenciador_2_con_evidencia}
3. {diferenciador_3_con_evidencia}

--- 4. TALK TRACKS ---

Cuando el prospecto dice: "{frase_tipica_1}"
Respuesta: "{respuesta_con_evidencia}"

Cuando el prospecto dice: "{frase_tipica_2}"
Respuesta: "{respuesta_con_evidencia}"

Cuando el prospecto dice: "{frase_tipica_3}"
Respuesta: "{respuesta_con_evidencia}"

--- 5. PREGUNTAS TRAMPA ---
{Preguntas que exponen debilidades del competidor sin atacar directamente}

1. "{pregunta_sobre_area_debil_1}"
   Lo que revela: {por_que_esta_pregunta_nos_beneficia}

2. "{pregunta_sobre_area_debil_2}"
   Lo que revela: {por_que_esta_pregunta_nos_beneficia}

3. "{pregunta_sobre_area_debil_3}"
   Lo que revela: {por_que_esta_pregunta_nos_beneficia}

--- 6. WIN/LOSS INSIGHTS (datos CRM) ---
Ganadas contra {competidor}: {n} deals ({revenue_total} EUR)
  {empresa_1}: {revenue} EUR - {que_hicimos_bien}
  {empresa_2}: {revenue} EUR - {que_hicimos_bien}

Perdidas contra {competidor}: {n} deals ({revenue_total} EUR)
  {empresa_3}: {revenue} EUR - Motivo: {razon_perdida}
  {empresa_4}: {revenue} EUR - Motivo: {razon_perdida}

Win rate vs {competidor}: {ganadas}/{total} ({porcentaje}%)
Patron: {insight_del_analisis_win_loss}

--- 7. CUANDO GANAMOS / CUANDO PERDEMOS ---
Ganamos cuando:
  - {escenario_favorable_1}
  - {escenario_favorable_2}
  - {escenario_favorable_3}

Perdemos cuando:
  - {escenario_desfavorable_1}
  - {escenario_desfavorable_2}
  - {escenario_desfavorable_3}
=============================================
```

---

## Paso 3: Guardar Battlecard (opcional)

Si el usuario lo pide, guardar la battlecard en `references/competitor_landscape.md` para futuras consultas.

---

## Success Criteria

- [ ] Overview del competidor con datos verificados
- [ ] Fortalezas y debilidades objetivas
- [ ] Tabla de diferenciacion con areas concretas
- [ ] Al menos 3 talk tracks con respuestas preparadas
- [ ] Al menos 3 preguntas trampa con justificacion
- [ ] Datos win/loss del CRM de Odoo (si existen)
- [ ] Patron "ganamos cuando / perdemos cuando" identificado
