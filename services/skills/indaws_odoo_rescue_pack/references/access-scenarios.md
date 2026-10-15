# Los tres escenarios de acceso (mejor → peor)

El Rescue Pack ya no exige "código + BD". Lo que se puede afirmar depende del acceso disponible.
En Fase 0 se **detecta el escenario** y el resto del flujo se adapta. La certeza del entregable
decrece 1 → 3, y hay que **decirlo en el informe** (no vender caso 3 como caso 1).

| Caso | Acceso | Qué se puede hacer | Motor | Puede "reventar" |
|------|--------|--------------------|-------|------------------|
| **1** (mejor) | BD + **código fuente** local | Salud de código real + Studio/acciones por SQL | rescue-pack (flujo clásico) | n/a (BD local) |
| **2** (hardcore) | **BD local** restaurable, **sin fuente** legible (`.pyc`) | Estático `.pyc`, sandbox ejecutable, inferencia de fórmula, diff vs Odoo limpio + Studio/acciones por SQL | **delega en `odoo-reverse-engineering` (modo backup)** | **Sí** (copia desechable) |
| **3** (peor) | **BD productiva** del cliente, solo lectura | Metadatos + muestreo de comportamiento + `ir.logging` pasivo. Sin salud de código | **delega en `odoo-reverse-engineering` (modo API)** | **NO** (producción) |

## Detección en Fase 0

Preguntar y clasificar antes de arrancar:
1. **¿Hay código fuente** de los módulos custom (git/zip/carpeta/servidor)? → si **sí** y hay BD ⇒ **Caso 1**.
2. Si **no** hay fuente: **¿la BD es una copia local** que podemos restaurar y ejecutar aparte, o es
   **producción del cliente**?
   - Copia local desechable ⇒ **Caso 2**.
   - Solo producción ⇒ **Caso 3**.
3. Un mismo proyecto puede ser **mixto** (parte con fuente, parte sin): aplica caso 1 a los módulos
   con fuente y caso 2/3 al resto. Dilo en el informe módulo a módulo.

**Puerta de arranque (sustituye al antiguo bloqueo):** hace falta **al menos acceso de lectura a la
BD**. Sin ninguna BD no hay Rescue Pack. En **Caso 3 es OBLIGATORIO el consentimiento escrito del
cliente** antes de conectar (lectura sobre su producción) — sin él, PARAR.

## Puerta de migración (Fase 0, ortogonal al escenario)

Al inicio se pregunta **¿ofrecer migración al cliente?** (`{ofrecer_migracion}`). Es independiente del
escenario 1/2/3 y condiciona:

| Elemento | `{ofrecer_migracion}` = sí | = no |
|---|---|---|
| Versión objetivo | Se pide (o última estable) | No se pregunta |
| Fase 2 EXTRA (Migrabilidad + 0-10 dificultad) | Sí | **Omitir** |
| Fase 4 Op.2 (migrar) | Sí (es la Op.2) | **Omitir** — propuesta solo Op.1 |
| Sección 07 (coste de migrar) | Sí | **Omitir** |
| `odoo-database-analyzer` + `odoo-version-api-reference` | Se ejecutan | **No se ejecutan** |

El diagnóstico del estado actual (secciones 01-06 salvo Op.2) se produce igual en ambos casos.

## Cómo delega en `odoo-reverse-engineering` (casos 2 y 3)

Delegación **horizontal por nombre** (no se copia código de esa skill). rescue-pack orquesta; RE hace
la reconstrucción sin fuente **hasta `consolidate.py`** y devuelve su `<out_dir>` (facts, behavior,
analysis, formulas, static, consolidated). rescue-pack integra ese JSON en su informe.

**Renderizador único:** el PDF lo monta SIEMPRE rescue-pack (Fase 5, `report_template.md` +
dynapps-brand). **NO se ejecuta el `build_report.py` de RE** — si se hiciera, saldría un PDF con otra
maqueta y otra marca (Indaws por defecto) y la forma dejaría de ser consistente entre los 3 casos.
Así el entregable es idéntico en forma (portada, 6 secciones, marca Dynapps); solo cambia el contenido.

- **Caso 2 → RE modo backup restaurado.** Cargar `odoo-reverse-engineering` y correr su flujo con
  filesystem + sandbox: `static_pyc.py` (imports/SQL/overrides/licenciado), `analyze_behavior.py`,
  `infer_formula.py`, `baseline_diff.py`, y el sandbox instrumentado. Aquí **sí se levanta Odoo** sobre
  la copia desechable (el `NEVER levantar Odoo` de rescue-pack aplica a casos 1 y 3, no al 2).
- **Caso 3 → RE modo API.** Cargar `odoo-reverse-engineering` en modo solo-lectura por API:
  metadatos, `analyze_behavior.py` (muestreo sin persistir) y `read_logging.py` (`ir.logging` pasivo).
  Nada de sandbox ni escrituras. Requiere el consentimiento escrito ya confirmado en Fase 0.

En ambos casos, la **extracción de Studio/acciones** propia de rescue-pack (`extract_studio.sh`,
`db_state.sh` por SQL) sigue corriendo donde haya BD: en caso 2 sobre el Postgres de la copia; en
caso 3 sobre lectura SQL/API de producción.

## Sub-caso: dump-only SIN addons (RE no aplica → reconstrucción por SQL)

`odoo-reverse-engineering` es **100% ORM/API**: necesita un Odoo vivo (modo API contra producción) o
levantable (modo backup sobre el filesystem `.pyc`). Hay una situación real que no cubre: **solo
tenemos el dump de la BD, sin los addons/`.pyc` y sin acceso live**. No se puede arrancar Odoo (faltan
dependencias) ni conectar por API. Ahí el motor es la **reconstrucción por SQL propia de rescue-pack**,
que no necesita Odoo:

- `scripts/reconstruct_custom.sh` → capa declarativa completa por módulo custom (modelos, campos con
  ttype/relation/related/compute/store/depends + selection, vistas `arch_db`, seguridad, menús,
  acciones+código, crons, informes QWeb, plantillas correo) en `custom_reconstruct/<mod>/FOOTPRINT.md`.
- `scripts/db_intelligence.sh` → dimensionado sin código: grafo de dependencias, versión/estado,
  complejidad por módulo, **fill-rate** (vivo vs muerto), última actividad, matriz de seguridad.

Recupera ~toda la capa **declarativa**; queda opaco solo el **Python imperativo** (compute/create/
write/onchange, controladores) — `ir_model_fields.compute/store` marca dónde. Si más adelante
aparecen los addons o un entorno live, se complementa con RE para subir la certeza del comportamiento.

## Degradación del entregable (decisión: explícita y honesta)

La salud de código clásica (Pilar A, secciones 03/05) necesita fuente. Sin ella, se degrada:

| Sección del informe | Caso 1 | Caso 2 (sin fuente, BD local) | Caso 3 (producción) |
|---|---|---|---|
| 03 Inventario / triaje | Completo (autor + fuente) | Por metadatos + `baseline_diff` vs limpio | Por metadatos (sin diff estructural fino) |
| 05 Salud de código | **Real** (auditoría de fuente) | **Parcial** vía estático `.pyc` (imports, SQL, overrides, licenciado); NO auditoría de calidad | **No evaluable sin fuente** (decláralo) |
| Reconstrucción funcional | Complemento | **Núcleo** (RE: 4 niveles de certeza) | **Núcleo**, con menor certeza |
| 02 Volumen BD / Studio | Completo | Completo (BD local) | Completo si hay lectura, si no parcial |
| 07 Coste de migrar (extra) | Sí | Sí, con incertidumbre marcada | Sí, con incertidumbre alta |

Regla de oro heredada de RE: **observar ≠ leer**. Una fórmula inferida o un comportamiento observado
nunca se presentan como "el código confirmado". El informe lleva el nivel de certeza por hallazgo.

## Sub-caso dump-only sin addons (RE no aplica)

Si solo hay el dump y **no** se puede levantar Odoo (sin filesystem/addons) ni hay API live,
`odoo-reverse-engineering` (100% ORM) queda descartada. El motor pasa a ser la **reconstrucción por
SQL de rescue-pack** (`reconstruct_custom.sh` + `db_intelligence.sh`, Fase 3): recupera la capa
declarativa completa del custom + dimensionado (fill-rate, dependencias) sin Odoo. Ver
`studio-extraction.md` §Reconstrucción por SQL.

## Detalle de los dos [STOP] de la Fase 0

`SKILL.md` los nombra en corto; aquí está lo que hay que preguntar y qué activa cada respuesta.

### [STOP] 1 — ¿se le ofrece migración al cliente? (`{ofrecer_migracion}`)
- **Sí** → pedir `{version_objetivo}` **y `{pct_migracion_recurrente}`** (por defecto el de
  `config.json`; sólo se cambia si el consultor indica otra cosa). Activa: Fase 2 eje de
  Migrabilidad, Fase 4 Opción 2, sección 07, y las skills `odoo-database-analyzer` /
  `odoo-version-api-reference`.
- **No** → se omiten la sección 07, la Opción 2 y esas dos skills; la propuesta queda sólo con la
  Opción 1. El diagnóstico del estado actual se produce igual (tabla arriba).

### [STOP] 2 — confirmar antes de arrancar
- **(a) Escenario** 1/2/3 y sus límites, dichos en voz alta.
- **(b) Caso 3 → consentimiento escrito**. Sin él, PARAR: es lectura sobre la producción del cliente.
- **(c)** ¿Se entrega el módulo `odoo_studio` consolidado, o sólo el diagnóstico?
- **(d) `{detalle}`**: `ejecutivo` (sólo documento A) o `completo` (A + anexo técnico con
  `file:line`). **Por defecto `completo`**: gerencia lee A, el técnico del cliente lee B.
- **(e) `{en_produccion}`**: ¿opera ya, o es implantación pre-arranque? Reencuadra la sección 2 del
  informe — ver `deliverable-structure.md` → *Encuadre*.

### Datos que hay que recoger en la Fase 0
Versión Odoo origen `{version}` y edición (CE/EE) · `{paths}` de los addons custom (caso 1) o ruta de
addons del backup (caso 2) · acceso a la BD · notas de la sesión de consultoría funcional (alimentan
la sección 06).
