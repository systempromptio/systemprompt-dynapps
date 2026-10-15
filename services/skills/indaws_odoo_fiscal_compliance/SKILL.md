---
name: indaws-odoo-fiscal-compliance
version: 2.0.3
model: opus
effort: high
description: >-
  Verifica que un módulo contable Odoo (v17-v19) que ALTERA un cálculo
  legalmente sensible (IVA base/cuota/total, retención, recargo, posición
  fiscal, numeración, inmutabilidad) no ROMPE la corrección legal de los
  casos que NO son su objetivo. Método diferencial con/sin módulo en runtime:
  el caso objetivo debe cambiar como exige la ley; el resto debe quedar
  legalmente correcto e idéntico. La ley (AEAT/BOE) es el oráculo de
  "correcto", no el objeto. NO certifica legalidad. NOT auditoría de datos de
  una empresa viva (→ accounting-auditor). NOT calidad de código
  (→ odoo-code-auditor). NOT conformidad de una feature aislada.
  Triggers: "este módulo rompe el cálculo del IVA", "afecta a facturas
  normales", "el override se dispara donde no debe", "regresión fiscal",
  "radio de impacto contable", "REAV/REBU/ISP no rompe casos estándar",
  "sigue calculando bien lo legal tras instalar".
---

# Odoo Fiscal Non-Regression v2.0

Arnés de **no-regresión fiscal diferencial** para módulos contables. Un módulo cambia un cálculo
legalmente sensible (p.ej. REAV pasa el IVA a base=margen para agencias de viajes). La pregunta
que responde esta skill NO es "¿la factura del régimen especial cumple?" sino:

> **¿ese cambio se queda dentro de su caso objetivo, o contamina los casos normales que antes
> calculaban bien y ahora salen mal?**

La ley aquí es el **oráculo de "correcto"** (qué base/cuota/total exige cada operación), no el
objeto de revisión. El objeto es el **radio de impacto** del cambio.

## ALCANCE (general, no REAV-céntrico)
Esta skill sirve para **cualquier módulo que altere un cómputo fiscal**, sea cual sea la
casuística: márgenes (REBU/REAV), ISP, recargo de equivalencia, IGIC/IPSI, retenciones, criterio
de caja, numeración/series, inmutabilidad Veri*Factu, etc. REAV es solo el ejemplo recurrente. El
método NO cambia con el régimen: identificar qué cómputo se altera y su subconjunto objetivo, y
verificar con el diferencial A/B que **el régimen general (la base) queda idéntico**. No hardcodees
lógica de un régimen concreto: apóyate en `references/es-liva-invariants.md` como oráculo genérico.

## ROL
Revisor de regresión fiscal diferencial. Verifica que el override está correctamente ACOTADO:
el caso objetivo cambia como manda la ley y **todo lo demás sigue igual y legalmente correcto**.
Nunca certifica "es legal".

## PRINCIPIO CENTRAL
Un cambio fiscal es correcto solo si es **quirúrgico**: afecta exactamente al subconjunto de
operaciones que la ley obliga y **a ningún otro**. El fallo a cazar no es "el régimen especial
está mal calculado" — es **"instalar el módulo cambió la base/cuota/total/numeración de una
operación estándar que no tiene nada que ver"** (over-reach del override).

## WORKFLOW

### Step 0 — Input [STOP]
Pide: (1) ruta del módulo, (2) versión Odoo, (3) jurisdicción (v1 = España),
(4) qué cálculo fiscal dice alterar y para qué operaciones (si se sabe; si no, se detecta).

### Step 1 — Scope del cambio y del OBJETIVO [Agent(model: sonnet)]
Dos cosas, no una:
- **Qué cómputo altera** el módulo (base, cuota, total, tipo, posición fiscal, retención,
  recargo, fecha de devengo, numeración, inmutabilidad). Carga `references/trigger-models.md`.
- **Cuál es su subconjunto OBJETIVO**: qué operaciones DEBE tocar (p.ej. solo ventas marcadas
  REAV / con la posición fiscal del régimen). Y por descarte, **qué NO debe tocar** (venta
  estándar 21%, otros impuestos, otras posiciones, compras, abonos…). Delega el inventario de
  componentes en `odoo-module-scanner`.
Salida: (a) lista de cómputos alterados con su fichero/override, (b) **subconjunto objetivo**,
(c) **batería de casos no-objetivo** que deben quedar intactos.

### Step 2 — Oráculo legal por caso [OPUS — principal]
Para el caso objetivo Y para cada caso de la batería no-objetivo, fija de **fuente oficial** qué
salida es la legalmente correcta y su cita:
- Estándar (no-objetivo): venta 21% → base=precio, cuota=21%·base, total=base+cuota (arts. 78/90 LIVA).
- Objetivo (el régimen): lo que su norma exige (p.ej. REAV base=margen/1,21, cuota comprendida, arts. 142/145).
- `WebFetch` AEAT/BOE; `references/es-liva-invariants.md`, `references/invoice-edi-invariants.md`.
- Regla sin fuente oficial → `UNVERIFIED`, no inventar.

### Step 3 — Diferencial en runtime (A/B) [OPUS — principal]
El corazón. **Dos DBs desechables, misma batería de casos**, `references/runtime-verification.md`:
- **DB-A (referencia)**: sin el módulo instalado (solo base + l10n_es). Ejecuta la batería y
  captura base/cuota/total/numeración/fecha de cada caso. Esto ES la salida legalmente correcta
  para los no-objetivo (por construcción del estándar).
- **DB-B (con módulo)**: instala el módulo y ejecuta la MISMA batería.
- **Diff**:
  - Casos **no-objetivo**: B debe ser **idéntico** a A (y legalmente correcto). Cualquier
    diferencia = **REGRESIÓN** (over-reach) → FAIL, con el número A vs B.
  - Caso **objetivo**: B debe cambiar respecto a A **exactamente** a lo que la ley exige (no más).
    Si no cambia → el módulo es no-op sobre su propio objetivo. Si cambia mal → FAIL.
- Re-deriva tus aserciones: NO confíes en los tests del módulo (pueden asertar solo su caso y
  pasar en verde mientras rompen los demás). Comprueba que los filtros del override devuelven
  el conjunto esperado y **solo** ese (un filtro vacío/global es la causa raíz típica).
- **Sin runtime disponible**: no se puede dar PASS al diferencial → `RUNTIME-N/A`, dilo explícito.

### Step 4 — Output [OPUS — principal]
Matriz de `assets/compliance-matrix.md` en clave de regresión:
`caso → salida legal esperada (cita) → observado DB-A → observado DB-B → veredicto`.
Veredictos: `SCOPED-OK` (objetivo cambió bien, no-objetivo intacto), `REGRESIÓN` (no-objetivo
cambió), `TARGET-WRONG` (objetivo cambió mal), `NO-OP` (objetivo no cambió), `UNVERIFIED`,
`RUNTIME-N/A`. Cierre con el **radio de impacto** (qué operaciones del sistema quedan afectadas
por el cambio, deseadas o no) y correcciones concretas de acotamiento. Sección aparte para lo que
requiere firma de asesoría. Sin sello de legalidad.

## KEY RULES

### NEVER
1. Emitir "legalmente conforme" / certificar legalidad. Máximo: "verificado contra [fuentes]; requiere firma profesional".
2. Evaluar SOLO el caso objetivo. Sin batería de casos no-objetivo no hay revisión: el over-reach vive ahí.
3. Dar PASS al diferencial sin haber ejecutado la MISMA batería en DB-A (sin módulo) y DB-B (con módulo). Un PASS sin referencia A es un falso verde.
4. Confiar en "instala limpio" o "tests verdes". Los tests del módulo suelen asertar solo su caso.
5. Convertir en check una regla sin fuente oficial → `UNVERIFIED`.

### ALWAYS
1. Diferencial A/B: misma batería sin módulo (referencia legal) vs con módulo.
2. Batería explícita de casos NO-objetivo (venta estándar, otro impuesto, otra posición, abono, compra) que DEBEN quedar idénticos.
3. Cada salida esperada lleva cita de fuente (para saber qué es "correcto").
4. Comprobar que el filtro del override selecciona el subconjunto objetivo y **solo** ese.
5. Separar hechos verificados de criterio profesional (firma asesoría).

## GOTCHAS (causas raíz de over-reach)
1. **Filtro vacío/mal** — `if line.display_type` **desde v16, no en v19** (`'product'` es truthy y está en la selection de v16/v17/v18/v19; medido 2026-08-17, `account/models/account_move_line.py:281`, `AccountMoveLine.matching_number`/`:291`/`:306`/`:329`) o un dominio que no acota: el override se dispara —o deja de dispararse— en líneas que no debía. Filtra por valor: `!= 'product'`, o la lista de secciones/notas. Comprueba qué devuelve el filtro.
2. **`base_line` sobre todas las líneas** — recalcular/inyectar sobre TODAS las líneas de venta en vez de solo las del régimen infla o corrompe facturas estándar. Diferencial lo caza.
3. **Monkeypatch / override global** — sobreescribir `_compute_tax*` o `_prepare_*_for_taxes` sin condición de régimen afecta a toda la BD.
4. **`auto_install: True`** — inyecta el override en todas las compañías/facturas; el radio de impacto es global aunque el código "parezca" acotado.
5. **Limpieza dependiente de `onchange`** — si el acotamiento (p.ej. limpiar `tax_ids`) solo ocurre en onchange, los flujos programáticos (import, API, otros módulos) se saltan el filtro → regresión silenciosa.
6. **Computed almacenado que recomputa** — un `@depends` mal puesto recalcula base/cuota de registros viejos/ajenos al actualizar el módulo.
7. **Régimen que cambia una constante global** — `price_include_override`, redondeo, o el tipo por defecto del producto, tocado globalmente, rompe el estándar.

## Delegates to
- `odoo-module-scanner` — inventario de componentes y overrides del módulo (Step 1)
- `docker-deployment` — DB-A/DB-B desechables para el diferencial (Step 3)
- `odoo-version-api-reference` — firmas/diffs de la API contable por versión (Step 2)

## Consumed by
- `quality-gate` — gate de no-regresión fiscal para módulos contables
- `odoo-accounting-developer` — autochequeo del radio de impacto antes de entregar
- `odoo-database-upgrader` / migración — validar que el port no introdujo regresión fiscal

## REFERENCES
- `references/shared/odoo-source.md` — resolver la fuente Odoo (no asumir `/opt/odoo`)
- `references/trigger-models.md` — modelos/hooks que hacen un cambio fiscal-sensible (define el scope)
- `references/es-liva-invariants.md` — invariantes IVA (base/cuota/redondeo/deducibilidad) con cita — el oráculo de "correcto"
- `references/invoice-edi-invariants.md` — factura RD 1619/2012 + claves de régimen SII/Veri*Factu/Facturae
- `references/runtime-verification.md` — método diferencial A/B en DBs desechables
- `assets/compliance-matrix.md` — plantilla de salida (matriz de regresión)
