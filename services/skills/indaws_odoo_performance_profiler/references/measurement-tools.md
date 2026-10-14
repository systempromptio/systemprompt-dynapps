# Measurement Tools (Odoo v17-v19)

Como medir de verdad. Elige la tecnica segun el caso y usa la MISMA en baseline y
verificacion. Anota siempre numeros concretos (queries, ms, picos de RAM/CPU).

> Para conectarte/ejecutar en una instancia viva, usa `odoo-pilot` (no escribas
> conexion ad-hoc). Para firmas de API de cada version, `odoo-version-api-reference`.

---

## 1. Modo dev y logging SQL

Arranca el servidor con instrumentacion:

```bash
odoo-bin -d <db> --dev=all --log-level=debug_sql 2>&1 \
  | grep -v 'unknown parameter.*export\|selection attribute will be ignored\|Will use the Wkhtmlto'
```

- `--dev=all`: recarga, sin cache de assets/QWeb, mejor para reproducir.
- `--log-level=debug_sql`: vuelca cada SQL con su tiempo. Cuenta SELECTs repetidas
  (N+1) y mide la mas lenta.
- Filtra el ruido habitual con el `grep -v` (regla de entorno).

Para aislar una request: limpia el log, reproduce UNA accion, cuenta queries.

---

## 2. Contador de queries en shell

Mide el coste de un metodo directamente:

```bash
odoo-bin shell -d <db> --no-http
```

```python
from odoo.tools import profiler  # disponibilidad segun version
recs = env['model'].search([...], limit=1000)
env.cr.sql_log_count = 0  # o usa el contador interno del cursor segun version
t0 = time.time()
recs.mapped('field_relacional.subfield')  # la operacion a medir
print('ms:', (time.time()-t0)*1000)
```

Patron fiable cross-version: envuelve con `env.cr` y compara `len` de queries antes
y despues, o usa el profiler integrado (`odoo.tools.profiler.Profiler`) que registra
queries + frames Python. Verifica el API exacto contra `odoo-version-api-reference`.

---

## 3. `assertQueryCount` (el patron de oro para fijar mejoras)

En un test (`TransactionCase` con `@warmup`):

```python
from odoo.tests.common import TransactionCase, warmup

class TestPerf(TransactionCase):
    @warmup
    def test_no_n1(self):
        recs = self.env['model'].search([], limit=100)
        with self.assertQueryCount(__system__=5):  # o sin clave segun version
            recs.mapped('partner_id.country_id.name')
```

- `@warmup` ejecuta una pasada previa para llenar caches; mide la 2ª pasada (la
  representativa). SIN warmup cuentas queries de registry/prefetch que no se repiten.
- Pon el limite igual al baseline medido; si baja tras el fix, ajusta y queda como
  test de regresion. Generalo/actualizalo con `odoo-test-generator`.

---

## 4. py-spy (perfilado de CPU, no de queries)

Cuando el cuello es CPU/Python y no BD (computos pesados, render):

```bash
pip install py-spy
# flame graph de un proceso Odoo en marcha:
py-spy record -o profile.svg --pid <PID_odoo> --duration 30
# top en vivo:
py-spy top --pid <PID_odoo>
```

Identifica las funciones que mas CPU consumen. Util para distinguir "lento por BD"
(no aparece CPU alta) de "lento por Python" (funcion concreta domina el flame).

---

## 5. Profiler integrado de Odoo (`@profile` / Profiler)

Odoo trae un profiler que combina SQL + frames Python:

```python
from odoo.tools.profiler import Profiler
with Profiler(collectors=['sql', 'traces_async']) as p:
    env['model'].costly_method()
# p.json() / volcado segun version
```

Tambien hay activacion via `--dev` y el menu de profiling en UI (Ajustes ->
Tecnico) en versiones recientes. Confirma collectors disponibles por version con
`odoo-version-api-reference`.

---

## 6. Perfilar requests / informes

- **Request HTTP/RPC:** activa el profiler de request (header/param segun version) o
  reproduce con `--dev=all` y cuenta queries del log para esa URL.
- **Informe QWeb:** mide el render aislando `_get_report_values` y el paso a PDF.
  Cuenta queries durante el render; un informe sano no debe escalar queries con el
  numero de lineas. Si escala, hay N+1 en la plantilla (ver antipatron 9).

```bash
# generar el informe por shell y cronometrar
odoo-bin shell -d <db> --no-http
```
```python
report = env.ref('module.report_xmlid')
t0=time.time(); report._render_qweb_pdf('module.report_xmlid', docids); print((time.time()-t0)*1000,'ms')
```

---

## 7. `EXPLAIN ANALYZE` para queries concretas

Cuando sospechas indice faltante o dominio costoso:

```sql
EXPLAIN ANALYZE SELECT ... FROM model WHERE campo = '...';
```

- `Seq Scan` sobre tabla grande -> candidato a `index=True`.
- `Index Scan` -> ya usa indice; el problema esta en otro sitio.
- Sorts/hash grandes -> revisa `order` y volumen.

---

## Checklist de medicion

1. Define la operacion exacta a medir (metodo, request, informe).
2. Captura baseline con UNA tecnica (queries + ms).
3. Aplica el fix.
4. Re-mide con LA MISMA tecnica.
5. Si no hay mejora medible -> revierte. Si hay -> fija con `assertQueryCount`.
