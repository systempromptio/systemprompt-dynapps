# Template: Email semanal al comercial

Plantilla de referencia para el paso 5 (emails comerciales). Un email por comercial
con >= 3 opp en pipeline. Todos los datos provienen de `scripts/analisis_deadlines.py`
y del JSON de `calculate_crm_health.py`; nunca se inventan nombres ni cifras.

Marca Indaws: tono tu, sin emojis, argumentar cada recomendacion, vocabulario
"reto / inversion / propuesta de valor". Omitir los bloques sin contenido (excepto
el de cierre de commit, que siempre lleva la pregunta).

---

Asunto: {nombre_comercial}, tu foco comercial para esta semana

Hola {nombre_comercial}:

Tu CRM esta {estado} ({score}/100). {linea_lectura: que es lo que mas te resta puntos
esta semana, en una frase con dato real}.

**Cierra esta semana**

Tu commit del mes (opp con +70% de probabilidad que cierran en {mes_actual}):

{por cada opp del commit: - {nombre_opp} — {importe} EUR ({prob}%), etapa {etapa}}
Total comprometido: {total_commit} EUR ({n} deals).

Dime antes del viernes que te comprometes a cerrar esta semana de esta lista.

**No dejes que se enfrien**

{si hay negociaciones en riesgo:}
En negociacion y enfriandose (es donde mas se pierde):
{por cada una: - {nombre_opp} ({importe} EUR, {prob}%) — {motivo}. {accion concreta}.}

{si hay En analisis sin fecha o vencida:}
Fechas de entrega a confirmar:
{por cada una: - {nombre_opp}: {SIN FECHA -> ponle fecha | VENCIDA hace Nd -> reprograma}.}

**Higiene que te penaliza**

{si hay actividades vencidas:}
Tienes actividades vencidas (el cliente lee el silencio como desinteres):
{por cada una: - {nombre_opp}: {dias}d de retraso — {resumen_actividad}.}

{si hay opp sin actividad futura:}
Estas opp no tienen ninguna actividad programada; toda opp activa debe tener
siguiente paso: {lista corta}.

**Demos**

{si hay demos agendadas esta semana:}
Tus demos de esta semana: {por cada una: - {nombre_opp} el {fecha}.}

{si hay demos sin avance >14d:}
Demos que hiciste hace tiempo y siguen sin avanzar (recuperalas):
{por cada una: - {nombre_opp}: {dias}d desde la demo, sigue en {etapa}.}

Cualquier cosa, me dices.

Un abrazo,
Victor
