# Plantilla: Ficha de entorno

Vista 360º de un único entorno. **Datos:** localiza el `id` por hostname/`url`/`database`, luego una lectura de
`database.env` (con `env_data`) + una de `database.env.module.version` filtrada por ese entorno para los cores.
Reglas en `report-rules.md`; umbrales en `metrics-and-thresholds.md`.

## Formato de salida

```
# 🏢 [hostname] — Ficha de entorno
*A fecha [DD/MM/AAAA]*

## Identidad
| Campo | Valor |
|---|---|
| Hostname | cliente-x.indaws.cloud |
| Base de datos | master |
| UUID | 02512768-… |
| Versión / edición | 18.0+e · enterprise |
| Hosting | Indaws |
| Estado | connected |

## Licencia (Enterprise)
| Caduca | Estado |
|---|---|
| 16/07/2026 | 🟠 Caduca en 28 d |

## Actividad (churn)
| Última actividad | Inactividad | Estado |
|---|---|---|
| 02/04/2026 | 77 d | 🟠 En riesgo |

*Desglose (`env_data['activity']`): presencia [fecha] · comentario [fecha] · (contexto: login [fecha], email [fecha],
escritura backend [fecha] en [busiest_model]).*

## Tamaño
| Usuarios | Módulos |
|---|---|
| 33 | 295 |

## Cores inDAWS
[N] cores. Por categoría: inDAWS Core [n] · Core Enterprise [n] · Connectors [n] · Inbuilt [n].
Lista (technical_name): indaws_base, indaws_mail, indaws_security, …
*Footprint de código custom: [X] líneas (LOC) en [N] módulos con dato — [suma de `loc_total`, receta "Footprint de
código de un entorno"]. Omite esta línea si ningún módulo custom del entorno tiene aún `loc_total`.*

## Personalización
| Tipo | Nº |
|---|---|
| Campos Studio | 78 |
| Vistas Studio | 38 |
| Automations | 17 |
| Crons custom | 1 |

## Seguridad (env_data['security_audit'])
| Métrica | Manual | Studio | Total |
|---|---|---|---|
| Grupos | 1 | 0 | 1 |
| Reglas de registro | 0 | 0 | 0 (globales: 0) |

## Liveness
| Último heartbeat | Última sync | Estado sync |
|---|---|---|
| 17/06/2026 07:28 | 17/06/2026 07:28 | success |
```

## Reglas de contenido
- Licencia: solo si `enterprise`; si Community, escribe "Community (sin caducidad)".
- Actividad: si `last_activity_date = false` → ⚪ Sin dato (sonda no ejecutada), y omite el desglose.
- Cores: cuenta y lista los de categorías 1–4; no mezcles con Odoo/OCA/Terceros.
- Seguridad: usa los contadores de `security_audit` (`manual_groups`/`studio_groups`, `manual_rules`/`studio_rules`,
  `global_rules`). Si destaca mucha personalización de seguridad, coméntalo en una línea.

**Cierre:** ofrece profundizar (p. ej. "lista todos los módulos OCA de este entorno", "detalle de las reglas de
registro", "¿qué automations tiene?").
