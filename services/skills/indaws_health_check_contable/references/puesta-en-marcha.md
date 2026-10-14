# Puesta en marcha y cadena de ejecución

Dependencias: `pip install openpyxl weasyprint pypdf`. `pdftoppm` (poppler) para mirar páginas.

## Carpeta del cliente (`--out`)

Todo lo que escribe el motor va a una carpeta por cliente y corte, fuera de la skill (el motor se
niega si está dentro): `config.json`, `datos.pkl` (volcado de la contabilidad), `extract/` (modo
mcp), `resultado.json`, `usos.json`, `puntos.json`, `descartes.json` (lo escribe el consultor), el
HTML intermedio y los dos entregables. **Nada de esta carpeta se sube a un repo ni se comparte**
salvo el PDF y el Excel, y solo al cliente. Sugerencia: `~/health-checks/<cliente>/<aaaa-mm>/`.

## Parámetros

```bash
M=<dir de la skill>/motor
python3 $M/hc.py init --out DIR --cliente "Nombre, S.L." --corte 31/08/2026 --modo api \
        --url https://cliente.odoo.com --db base --user usuario [--cia 2]
```

`config.json` guarda `HC_CLIENTE`, `HC_CORTE_CORTO`, `HC_ODOO_MODO`, `HC_CIA`, `ODOO_URL`,
`ODOO_DB`, `ODOO_USER`. **Nunca la credencial.** El texto largo del corte («31 de agosto de
2026») se deduce; `HC_CORTE` lo sobrescribe si hace falta. Una variable exportada gana a
`config.json`. Otras opcionales: `HC_PROTOTIPO=1` (base de demostración), `HC_CATALOGO` (otro
Excel de catálogo; por defecto la versión más alta de `catalogo/`).

Sin `--cia` en una base con varias compañías, `carga` para y lista las compañías con su id.

## Modo api (odoo-pilot)

Mismas variables que `odoo-pilot`: `ODOO_URL`, `ODOO_DB`, `ODOO_USER`, `ODOO_KEY` (API key:
Preferencias → Seguridad de la cuenta → Nueva clave API). Si el consultor ya tiene el entorno de
odoo-pilot, `eval $(<odoo-pilot>/scripts/auth.sh)` deja además `ODOO_PROTOCOL`; si no, `conn.py`
detecta la versión (JSON2 en v19+, JSON-RPC antes).

El entorno no persiste entre comandos de Bash: la clave se pasa **en el mismo comando** que lanza
el paso y no se escribe en ningún fichero:

```bash
ODOO_KEY='…' python3 $M/hc.py carga --out DIR
```

Usuario con permisos de lectura en contabilidad; ideal, un usuario técnico de solo lectura.

## Modo mcp

Ver `modo-mcp.md`.

## Cadena

```bash
python3 $M/hc.py carga   --out DIR   # carga masiva -> datos.pkl (se recarga sola si cambia compañía, base o corte)
python3 $M/hc.py run     --out DIR   # guarda, capa 0, catálogo, aplicabilidad, chequeos -> resultado.json, usos.json
python3 $M/hc.py puntos  --out DIR   # fichas de cliente -> puntos.json, descartados.json
python3 $M/hc.py informe --out DIR   # HTML intermedio con marca Dynapps
python3 $M/hc.py pdf     --out DIR   # PDF con índice y marcadores
python3 $M/hc.py excel   --out DIR   # Anexo de evidencias
python3 $M/hc.py todo    --out DIR   # los seis seguidos (sin las paradas del SKILL.md)
```

`run` imprime los usos, los estados, la cobertura y los chequeos que han reventado (NO MIRADO con
su traza en `resultado.json`, campo `_traza`). `puntos` imprime cada punto y los huecos que han
salido como punto genérico.

## Descartes del consultor

Un hallazgo que al revisar la evidencia resulta no mover nada se descarta en
`DIR/descartes.json` y se vuelve a lanzar desde `puntos`:

```json
[{"n": 145, "motivo": "La contabilidad la lleva un único usuario técnico en este entorno, así que el dato no dice nada sobre el reparto real de funciones."}]
```

El motivo va en lenguaje de cliente: sale tal cual en el apartado 02.
