# Impacto Odoo y radio de explosión (Step 5)

Se comprueba **en el Docker del Step 1**, no se lee. Aplica en los dos modos: el de contrato tiene el
checklist del arquitecto; el de bug-fix no tiene MD, y por eso lleva un mínimo propio — sin él este paso
se quedaba vacío justo en el modo más frecuente.

---

## Modo CONTRATO

Repasar lo que el arquitecto marcó como APLICA en el MD, **verificándolo en el entorno**:

- datos existentes / migración (p.ej. registros del dump recalculados tras el `-u`),
- multi-company, multi-currency,
- seguridad y ACL,
- upgrade-safety,
- rendimiento.

## Modo BUG-FIX (sin MD)

### 1. Radio de explosión del módulo tocado

Determinista, cero tokens. Tres números:

- **dependientes directos y transitivos** dentro del monorepo,
- cuántos de ellos son **`auto_install`** — se instalan solos en cualquier BD que cumpla deps, así que
  el alcance real es todo el parque, no solo quien lo pida,
- qué **proyectos del host montan ese repo**.

> ⚠️ **Estos números caducan: MÍDELOS, no los cites.** Re-medidos el 2026-08-07 y ya habían derivado
> dos veces (la skill decía 75/160 y 13 proyectos; una pasada intermedia midió 76/162 y 17):
>
> | Magnitud | Skill (original) | 2026-08-07 | **2026-08-17** |
> |---|---|---|---|
> | manifests en `indawsodoo/18` | 160 | 162 | **164** |
> | de ellos `auto_install: True` | 75 | 76 | **76** |
> | proyectos del host que montan `indawsodoo/*` | 13 | 26 ⚠️ | **20 únicos** |
>
> Y hay **parque en cuatro majors**, no tres: `/16` 2 · `/17` 3 · `/18` 13 · `/19` 7. Por eso la
> pregunta "cuántos proyectos monta este repo" no tiene una sola respuesta — hay que decir **de qué
> major**, y si se responde al conjunto, **contar proyectos, no montajes**.
>
> ⚠️ **El 26 estaba mal, y estaba mal de una forma que conviene reconocer: es una SUMA de los cuatro
> majors** (2+3+13+8), no un conteo de proyectos. Los proyectos únicos son **20**, porque hay quien
> monta varios a la vez —`INDAWS_MCP` monta los cuatro, `AI_OCR` y `TRANSPORTISTAS` montan dos—.
> Sumar columnas de una tabla de montajes y llamar al resultado "proyectos" infla el radio de
> explosión y hace el aviso menos creíble, no más.
>
> Comandos para re-medir en el momento (segundos, 0 tokens) — **corregidos y ejecutados el
> 2026-08-17; dos de los tres anteriores no reproducían sus propios números** (ver la nota de abajo):
> ```bash
> P=${PROJECTS_ROOT:-/opt/PROYECTS}
> ls -d $P/indawsodoo/18/*/__manifest__.py | wc -l                                  # manifests: 164
> grep -lE '"auto_install"[[:space:]]*:[[:space:]]*True' $P/indawsodoo/18/*/__manifest__.py | wc -l
> # ...y si el repo mezcla comillas, la forma segura:
> grep -lE "[\"']auto_install[\"'][[:space:]]*:[[:space:]]*True" $P/indawsodoo/18/*/__manifest__.py | wc -l   # 76
>
> # proyectos UNICOS que montan un major (no ficheros, no montajes):
> grep -rlE "indawsodoo/18(/|[\"':]|$)" --include='docker-compose*.y*ml' --include='*.env' "$P/" \
>   | sed "s|^$P/||" | cut -d/ -f1 | sort -u | grep -v '^indawsodoo$' | wc -l        # 13
> ```
>
> **La trampa que hubo aquí, y es la lección de la caja entera:** este bloque se escribió para decir
> *"mídelos, no los cites"* y traía los comandos… **y dos de los tres devolvían otra cosa que la
> tabla**. `grep -l "'auto_install': True"` daba **1**, no 76 —el repo escribe
> `"auto_install": True` con comillas dobles en 75 de los 76 manifests—, y el de proyectos daba
> **10**, no 13, porque `$P/*/docker-compose*.y*ml` no entra en subdirectorios
> (`AI_OCR/.docker18/`, `INDAWS_MCP/staging-local*/`, `TRANSPORTISTAS/docker18/`, `Tecniman/_legacy/`)
> y además cuenta **ficheros**, no proyectos.
>
> **Un número viejo se ve; un comando roto parece rigor.** Si pones el comando al lado de la cifra,
> ejecútalo — o estás firmando la cifra dos veces.

Órdenes de magnitud medidos en `indawsodoo/18` (fechados, mismo aviso):

| Módulo tocado | Transitivos | De ellos `auto_install` |
|---|---|---|
| `indaws_base` | 136 de 160 | 70 |
| `indaws_product` | 29 | 20 |
| `indaws_account` | 21 | 14 |
| `indaws_sale` | 15 | 9 |
| `indaws_stock` | 6 | 5 |

Cálculo: leer los `__manifest__.py` del repo con `ast.literal_eval`, montar el grafo de `depends` y
cerrar transitivamente hacia arriba. Los proyectos afectados salen de grepear los `docker-compose.yml`
del host por la ruta del repo.

**Ese número va al feedback del PR.** Un fix "pequeño" en `indaws_base` no es pequeño.

En custom de cliente el radio es 1 proyecto — pero un `depends` nuevo hacia Core en el `__manifest__`
sí merece mirada.

### 2. Los otros caminos

El fix arregla su caso: ¿y los demás flujos que pasan por el código tocado? Es lo que separa un fix de
un parche.

### 3. El checklist estándar, aunque nadie lo haya marcado

Migración de datos, multi-company, ACL, rendimiento. Sin MD no hay quien lo marque, así que se repasa
igual.
