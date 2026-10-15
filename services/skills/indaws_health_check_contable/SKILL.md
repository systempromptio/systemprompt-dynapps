---
name: indaws-health-check-contable
version: 2.5.0
effort: high
model: opus
description: >-
  Ejecuta el health check contable mensual sobre el Odoo de un cliente (solo sociedades
  mercantiles, v17+): pregunta contra qué Odoo (odoo-pilot o un MCP conectado), aplica 177
  comprobaciones acotadas a lo que la empresa usa de Odoo y entrega un PDF con marca Dynapps y
  un Excel de evidencias registro a registro. Usar
  cuando pidan: health check contable, revisión de cierre mensual, informe de cierre para el
  cliente, qué hay que corregir en Odoo antes de cerrar el mes. NOT auditoría PGC interna por
  rango de fechas con informe Markdown → accounting-auditor.
---

# Health Check Contable

Revisión mensual del **uso del Odoo de un cliente**: comprueba que el sistema tenga los datos y
los enlaces que hacen falta para que su información sea fiable, y dice qué registro concreto hay
que tocar. Solo lectura sobre Odoo.

| Entregable | Responde |
|---|---|
| `health-check-contable-<cliente>.pdf` | **qué** hay que atender y **por qué** |
| `Anexo_Evidencias_<cliente>.xlsx` | **cuál**: cada asiento, cuenta, tercero y producto afectado |

La skill lleva catálogo (`catalogo/`), motor (`motor/`) y logos (`assets/`). El motor se lanza
con `motor/hc.py` desde cualquier directorio.

## Critical

- **Un falso verde es el único fallo que no tiene arreglo después.** Un chequeo que revienta es
  NO MIRADO, nunca LIMPIO; un LIMPIO sin universo es NO MIRADO. Ver `references/alcance-y-usos.md`.
- **Los datos del cliente viven en su carpeta (`--out`), nunca en la skill** ni en un repo:
  `datos.pkl` es un volcado de su contabilidad.
- **La credencial nunca se escribe en un fichero**: solo en `ODOO_KEY`, en el mismo comando.
- **Nada del cliente anterior**: todo sale de `config.json` y `descartes.json` de su carpeta.

## Workflow [opus, principal; sin subagentes]

**Paso 0 · Contra qué Odoo** [STOP]. Preguntar con AskUserQuestion:
- **Vía**: odoo-pilot (API directa; cualquier volumen) o un MCP de Odoo conectado en la sesión
  (listar los que exponen `search_read`/`execute_kw`; solo bases pequeñas, ver
  `references/modo-mcp.md`).
- **Instancia**: URL, base y usuario (odoo-pilot) o conector (MCP); **compañía** si hay varias;
  **fecha de corte** (dd/mm/aaaa); **nombre del cliente** tal como sale en el informe; **carpeta
  de trabajo** (por defecto `~/health-checks/<cliente>/<aaaa-mm>/`).

Con odoo-pilot, pedir la API key y usarla solo como `ODOO_KEY='…'` delante de cada comando. Con
MCP, hacer primero la guarda de volumen. Luego `hc.py init` (`references/puesta-en-marcha.md`).

**Paso 1 · Carga** [opus]. `hc.py carga`. En modo mcp, bucle de `extract.py` hasta que no
queden pendientes. Si avisa de topes alcanzados, parar y subir el tope.

**Paso 2 · Guarda, usos y chequeos** [opus]. `hc.py run`.
- Si la empresa no es mercantil (plantilla, o 72x/65x con movimiento real), **el motor para**:
  explicarlo y no entregar nada al cliente.
- [STOP] Enseñar al consultor la línea «Odoo se usa aquí para…», los usos ACTIVO / PARCIAL /
  INACTIVO, la cobertura y los chequeos que han reventado. Si corrige el propósito, se corrige
  el alcance.

**Paso 3 · Evidencia y descartes** [opus]. Revisar la evidencia de cada hueco contra
`references/falsos-positivos.md` y los saldos contra el balance de Odoo. Lo que no mueve nada va
a `descartes.json`. `hc.py puntos`: redactar «qué implica» y «qué hacer» de cada punto `generico`.

**Paso 4 · Entregables** [opus]. `hc.py informe`, `pdf`, `excel`. Pasar la verificación de
`references/informe-cliente.md`: renders mirados, grep de módulos, normas y rayas largas.

**Paso 5 · Entrega** [STOP]. Resumen al consultor: puntos por prioridad, cobertura, descartados,
NO MIRADO relevantes y ruta de los dos ficheros. El envío al cliente lo decide el consultor.

## GOTCHAS

1. **Impuestos y posiciones fiscales viven en la compañía madre**: filtrar por la hija devuelve
   cero y pinta verde. El motor usa la cadena de `parent_id`.
2. **Lo habitual es no regularizar la 129**: su saldo deudor es una recomendación, no un hallazgo.
   Nada del cierre se mira dentro del asiento de cierre. `references/cierre-y-129.md`.
3. **El diario de cierre no se quita de los saldos del balance**: ahí vive la regularización.
4. **Ejercicio fiscal, nunca natural**, salvo 347 y trimestres de IVA.
5. **Muchos chequeos aún no declaran universo**: en una base casi vacía salen LIMPIO. Revisar a
   mano los LIMPIO de usos con poco volumen antes de ponerlos en «lo que está correcto».
6. **Modo mcp**: nunca inventar ni vaciar una respuesta para avanzar; por encima de ~5.000
   apuntes, pasar a odoo-pilot.
7. Más trampas de Odoo 19 (`group_ids`, `analytic_distribution` JSON, planes sin `company_id`):
   `references/trampas-odoo19.md`.

## NEVER / ALWAYS

- NEVER escribir en el Odoo del cliente: el motor es de solo lectura.
- NEVER citar normas ni nombres técnicos de módulo en el informe; el efecto se explica por el sistema.
- NEVER entregar un punto `generico` sin redactar ni un saldo sin contrastar con el balance de Odoo.
- ALWAYS cargar `dynapps-brand` antes de tocar el diseño del informe.
- ALWAYS declarar la cobertura sobre los usos activos y decir dónde no se ha podido mirar.

## Delegates to

- `odoo-pilot` (common): convenciones de conexión y `auth.sh` en la vía API.
- `dynapps-brand` (common): identidad del informe.

> Note: `common:references/shared/` para el patrón de conexión; aquí lo implementa `motor/conn.py`
> en Python porque la carga masiva no cabe en llamadas sueltas de `search_records.sh`.

## Consumed by

Ninguna skill: se invoca directamente (consultor o dirección, antes del cierre mensual).

## REFERENCES

- `references/puesta-en-marcha.md`: parámetros, `hc.py`, cadena, descartes
- `references/modo-mcp.md`: bucle de extracción por MCP y guarda de volumen
- `references/alcance-y-usos.md`: estados, universo, guarda mercantil, capa 0, analítica
- `references/cierre-y-129.md`: regularización, resultado sin aplicar
- `references/trampas-odoo19.md`: trampas de modelo y de carga
- `references/falsos-positivos.md`: anexo de evidencias y purga de falsos positivos
- `references/informe-cliente.md`: texto, marca, estructura, PDF y verificación
- `CHANGELOG.md`: historial y pendientes conocidos
