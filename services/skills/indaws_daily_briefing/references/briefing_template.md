# Template: Briefing Comercial

## Formato de salida

```
======================================
  BRIEFING COMERCIAL - {fecha_hoy}
  {nombre_comercial}
======================================

--- PRIORIDAD #1 ---
{La accion mas importante del dia, basada en urgencia e impacto.
 Ejemplos: "Cerrar propuesta con [Empresa] (45.000 EUR, vence manana)"
           "Responder a [Empresa] - llevan 3 dias esperando"
           "Preparar demo de esta tarde con [Empresa]"}

--- NUMEROS DEL DIA ---
Pipeline activo:      {total_pipeline} EUR ({n_oportunidades} oportunidades)
Cerrado este mes:     {cerrado_mes} EUR ({n_cerradas} operaciones)
Reuniones hoy:        {n_reuniones}
Actividades vencidas: {n_vencidas}
Actividades hoy:      {n_hoy}

--- AGENDA ---
{hora} - {nombre_reunion} con {empresa/contacto}
         Contexto: {info relevante del CRM - etapa, revenue, ultima nota}
         Preparacion: {accion concreta}

{hora} - {nombre_reunion_2} ...

--- ALERTAS PIPELINE ---
[!] {empresa}: Oportunidad de {revenue} EUR estancada {dias} dias (etapa: {etapa})
[!] {empresa}: Vence en {dias} dias - {revenue} EUR - {probabilidad}%
[!] {empresa}: Actividad vencida desde {fecha}

--- TOP 5 ACCIONES DEL DIA ---
1. {accion_concreta_y_especifica}
2. {accion_concreta_y_especifica}
3. {accion_concreta_y_especifica}
4. {accion_concreta_y_especifica}
5. {accion_concreta_y_especifica}
======================================
```

## Reglas de Priorizacion

El algoritmo de prioridad sigue este orden:
1. **Actividades vencidas** (deuda de gestion - resolver primero)
2. **Oportunidades cerrando hoy/manana** (revenue inmediato)
3. **Reuniones del dia** (preparacion necesaria)
4. **Oportunidades de alto valor estancadas** (revenue en riesgo)
5. **Nuevos leads sin atender** (velocidad de respuesta)

## Modos de Ejecucion

| Modo | Descripcion | Uso |
|------|-------------|-----|
| Individual | Briefing para un comercial | "Dame el briefing de Victor" |
| Equipo | Resumen consolidado del equipo | "Briefing del equipo" |
| Rapido | Solo prioridad #1 y numeros | "Que es lo mas urgente hoy?" |
