# health-check-contable, historial

## 2.5.0 (2026-09-28): empaquetado como skill de management-skills

- **Elegir el Odoo al inicio**: modo `api` con las variables de odoo-pilot (`ODOO_URL`,
  `ODOO_DB`, `ODOO_USER`, `ODOO_KEY`; siguen valiendo `ODOO_HOST`/`ODOO_PWD`) y JSON2 en v19+, o
  modo `mcp`, con consultas servidas desde `extract/` y el bucle de `extract.py`.
- **Carpeta del cliente (`--out`)**: todo lo que se escribe va ahí, nunca al directorio de la
  skill. `hc.py` lanza la cadena desde cualquier sitio y guarda los parámetros en `config.json`
  (sin credencial).
- **`puntos.py` genérico**: fuera las cifras del primer cliente en títulos y textos; un hueco sin
  punto sale como punto propio en vez de quedarse en consola; los descartes pasan a
  `descartes.json` de la carpeta del cliente.
- `aplicabilidad.py`: la moneda extranjera se mide contra la de la compañía (estaba fijo el id 126).
- `carga.py` avisa si una consulta alcanza su tope de registros (`truncados`).
- `ratios_xlsx.py` y sus fórmulas salen de la skill: alimentaban una skill de ratios que no existe
  en el ecosistema. Pendiente de decidir si forman skill propia.
- SKILL.md reducido a índice; el detalle pasa a `references/`. README eliminado.

## 2.4

- Chequeo 13: saldos cruzados en todas las cuentas a cobrar y a pagar, también sin tercero.
- Estado RECOMENDACIÓN; el saldo de la 129 sin regularizar pasa a recomendación.
- Todo por ejercicio fiscal, nunca natural (salvo 347 y trimestres de IVA).
- `informe.py` sin textos del primer cliente; apartado 04 generado del punto con más peso.

## 2.3

- Compañía (`HC_CIA`), cadena de madres y corte fuera del código; `datos.pkl` se recarga si es de
  otra compañía, base o fecha.
- Chequeos 176 (nóminas pendientes) y 177 (retenciones de IRPF).

## 2.1 y 2.2

- El escenario habitual es no regularizar: chequeos 14, 24, 81 y 95 corregidos para ese caso.

## Pendientes conocidos

- **48 de 177 chequeos sin implementar**: salen NO MIRADO y bajan la cobertura (correcto).
- **Regla del universo sin retrofitar** en la mayoría de chequeos: en una base simulada vacía, 19
  chequeos salen LIMPIO. Es la tarea que protege contra falsos verdes.
- El modo api JSON2 está probado en detección, no contra una base real v19 completa.
