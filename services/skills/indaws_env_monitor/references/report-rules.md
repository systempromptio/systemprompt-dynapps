# Reglas de los informes

Convenciones comunes a todos los informes predeterminados. Los datos se leen con las recetas de `query-recipes.md` y
se clasifican con los umbrales de `metrics-and-thresholds.md`.

## Generales
- **Fecha de referencia:** hoy, del contexto de sesión. No la inventes ni la codifiques en la plantilla.
- **Solo lectura.** Nunca escribas en el registro. Nunca inventes entornos, campos ni cifras: si falta el dato, dilo.
- **Nombrar por hostname** (`display_name`), no por el `name` interno (`[UUID8] master`).
- **Clientes vs. master:** hay un autoregistro del propio master; si la pregunta es sobre clientes, exclúyelo. El
  parque son 68 registros (67 `connected` + 1 `pending`).
- **Idioma:** español de España. Fechas `DD/MM/AAAA`; inactividad/plazos como entero con unidad (`87 d`).
- **Una sola lectura por entorno** cuando necesites `env_data` (trae el JSON entero y navega las claves).

## Leyenda de estados
🔴 crítico · 🟠 riesgo · 🟡 atención · ✅ ok · ⚪ sin dato

## "Sin dato"
`last_activity_date = false` = entorno aún no auditado por la sonda → ⚪ **sin dato**, NUNCA "dormido". No concluyas
churn sin la señal; muéstralos aparte de los dormidos.

## Cierre (clave del diseño)
Cada informe predeterminado es un punto de partida. **Al final, recuerda al usuario que puede preguntar cualquier cosa**
sobre los entornos (filtrar, cruzar, profundizar) — la skill no se limita al informe: responde libre con el
data-dictionary y las recetas.
