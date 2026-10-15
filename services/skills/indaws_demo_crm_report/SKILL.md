---
name: indaws-demo-crm-report
description: >
  Genera un dashboard HTML con el estado del CRM de Odoo, incluyendo prediccion
  de ventas del mes (best/likely/worst), ranking de comerciales con CRM Hygiene
  Score (0-100), resumen por comercial con valoracion cualitativa del pipeline
  generada por IA, top deals, alertas (zombies/propuestas sin seguimiento/
  concentracion) y tendencia 4 semanas. Solo genera dashboard (no envia emails
  ni publica en Odoo website). Usar cuando el usuario pida: informe CRM,
  dashboard de ventas, forecast del mes, estado del pipeline, ranking
  comerciales, analisis de pipeline por comercial.
version: 1.0.2
model: sonnet
effort: medium
---

# Demo · CRM report

Dashboard HTML visual del estado del CRM con prediccion de ventas, ranking y analisis cualitativo por comercial.

## Prerequisitos

- **odoo-pilot** (common skill) para la extraccion de datos
- **Credenciales Odoo** en el entorno: `ODOO_URL`, `ODOO_DB`, `ODOO_USER`, `ODOO_KEY`
- **Python 3.8+** (sin librerias externas, solo stdlib)
- `config/config.json` relleno con objetivos mensuales por comercial y team_id

## Workflow

| Paso | Accion | Output |
|------|--------|--------|
| 1 | Autenticar en Odoo (`auth.sh`). Leer `config/config.json`. | -- |
| 2 | Ejecutar `python scripts/main.py --output output/report.html --kpis output/kpis.json`. El script extrae datos de Odoo, calcula 20+ KPIs y genera un HTML con placeholders `{{LLM_ASSESSMENT:<nombre>}}` y `{{LLM_ACTIONS:<nombre>}}` por comercial. | `report.html` + `kpis.json` |
| 3 | Leer `kpis.json`. Para **cada comercial**, generar: (a) una **valoracion cualitativa del pipeline** (3-5 lineas) que interprete los numeros y (b) **3 acciones concretas** para la proxima semana. | texto por comercial |
| 4 | Reemplazar los placeholders del HTML con las valoraciones y acciones generadas en paso 3. Guardar el HTML final. | `report.html` definitivo |
| 5 | Informar al usuario de la ruta del HTML y del resumen de hallazgos clave (3-5 bullets). | -- |

## Como debe ser la valoracion cualitativa (paso 3)

Tres o cuatro frases que interpreten los numeros, no que los repitan:
- Salud global del pipeline (distribucion por etapas, concentracion, diversificacion)
- Puntos fuertes y riesgos concretos (con nombres de deals si aplica)
- Proyeccion a fin de mes vs objetivo (en verde / ambar / rojo)

Ejemplos ilustrativos (varía estructura y énfasis según los datos):
> *"Pipeline solido y diversificado: 18 deals activos repartidos entre Cualificado (6), Propuesta (7) y Negociacion (5), sin concentracion peligrosa — el mayor deal representa el 19% del forecast. Hygiene 87 con minima friccion en datos. Punto de atencion: 2 propuestas sin tocar hace mas de 4 dias. Proyeccion a fin de mes por encima del objetivo (+8%)."*
> *"Casi todo el forecast depende de un deal de 60K en Negociacion sin actividad desde hace 9 dias; fuera de el solo quedan leads en Cualificado. Si ese deal se retrasa, el mes cierra un 40% por debajo del objetivo."*

Ejemplo incorrecto (solo repite numeros):
> ~~"Tiene 18 deals, forecast 115K, hygiene 87, objetivo cumplido 108%."~~

## Como deben ser las 3 acciones (paso 3)

Acciones concretas, ejecutables esta misma semana, referidas a deals especificos cuando se pueda. Ejemplo:
- "Retomar propuesta a Cerámicas Peñíscola (4 dias sin actividad, 85K)"
- "Programar actividad de seguimiento en 3 deals en Cualificado sin actividad"
- "Reactivar lead `Plásticos Romero` (zombie, 12 dias inactivo, 62K)"

## KPIs calculados (20+)

Detalle completo en `references/kpi_definitions.md`. Resumen:

**Executive**: Pipeline total (n + valor), Forecast best/likely/worst, Gap vs objetivo equipo, Status global.

**Por comercial**: Leads activos, Forecast ponderado mes, Ventas cerradas mes, Win rate 90d, Ticket medio, Hygiene Score (0-100), Cumplimiento objetivo %.

**Hygiene Score** (media ponderada 0-100):
- Completitud de datos (25%): % leads con revenue + deadline + probability
- Actividades al dia (25%): % actividades no vencidas
- Frescura (20%): % leads tocados ultimos 7 dias
- Tiempo de respuesta (15%): mediana horas lead creado → primera actividad
- Seguimiento propuestas (15%): % propuestas con actividad ultimos 3 dias

**Alertas**: Zombies (>7d inactivo), Propuestas sin seguimiento (>3d), Deals en riesgo, Concentracion (>50% del forecast en un deal).

**Tendencia**: Leads creados ultimas 4 semanas, mes vs mes.

## RULES

1. **NUNCA** hardcodees objetivos en codigo — van en `config/config.json`
2. **NUNCA** calcules KPIs manualmente — siempre con `scripts/main.py`
3. **SIEMPRE** genera primero el HTML + kpis.json y DESPUES rellena los placeholders con las valoraciones cualitativas
4. La valoracion cualitativa es **narrativa**, no una repeticion de numeros
5. Las 3 acciones deben ser **concretas y referidas a deals reales** (usar `top_deals_by_user` del JSON)
6. Usar solo la libreria estandar de Python — sin `pip install` en la demo

## GOTCHAS

1. El informe solo considera el `team_id` definido en config. Si el usuario quiere cross-team, extender `config/config.json`
2. `probability` en Odoo esta en 0-100, el script la normaliza a 0-1 para el forecast ponderado
3. `read_group` en Odoo 19 JSON2 exige kwargs (`domain`, `fields`, `groupby`), no positional args
4. Los stages son case-sensitive y se mapean por nombre exacto en `config.json`
5. Hygiene no penaliza con datos incompletos en etapa "Nuevo" (aun no cualificado)
6. Si un comercial tiene 0 deals activos, su hygiene se muestra como "N/A", no como 0

## Delegates to

- **odoo-pilot** (common): extraccion de datos de Odoo (crm.lead, sale.order, res.users, mail.activity, crm.stage)

## References

| File | Content |
|------|---------|
| `references/kpi_definitions.md` | Definicion y formula de cada KPI |
| `config/config.json` | Objetivos, pesos del Hygiene Score, umbrales, mapeo de stages |
