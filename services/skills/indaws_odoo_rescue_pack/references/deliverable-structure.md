# Estructura del entregable Rescue Pack

El Rescue Pack entrega **DOS documentos PDF de marca** (estándar acordado por el equipo), pensados
para **públicos distintos**. El nivel de detalle se decide en Fase 0 (`{detalle}`):

- `ejecutivo` → solo el Documento A.
- `completo` (**por defecto**) → Documento A **+** Documento B.

| Documento | Fichero | Público | Contenido |
|-----------|---------|---------|-----------|
| **A — Informe ejecutivo** | `rescue_pack_<cliente>.pdf` | Gerencia / decisor | Portada + 6 secciones. Se lee rápido, "hasta los importes". |
| **B — Anexo técnico** | `rescue_pack_<cliente>_anexo_tecnico.pdf` | Técnico del cliente / dev | Detalle módulo por módulo, flujos, bugs de runtime, **referencias a código (`file:line`)**. |

> **Regla de oro (adaptar la entrega al público):** el ejecutivo describe **qué está mal y cuánto
> cuesta arreglarlo**; el anexo describe **exactamente dónde y por qué** (código). El ejecutivo
> **referencia al anexo**; el anexo es autónomo pero se apoya en las notas del ejecutivo.

---

## Documento A — Informe ejecutivo (Portada + 6 secciones)

Núcleo = estado actual (2-3). Secciones **4 y 6 valoradas** (solo importe €; horas internas).
Plantilla: `assets/report_template.md`.

| # | Sección | Origen | Tipo |
|---|---------|--------|------|
| — | **Portada** | `dynapps-brand` | marca |
| 1 | **Introducción** (contexto; ver *encuadre* abajo) | Consultoría / Fase 0 | breve |
| 2 | **Estado del sistema** (según encuadre: BD en producción / composición del desarrollo) | Fase 3 — `db_state.sh` | núcleo |
| 3 | **Salud del código** (OCA / custom / Studio, salud 0-10, tabla resumen) | Fase 2 (`odoo-code-auditor`) | núcleo |
| 4 | **Mejoras a realizar — valorado, 2 niveles** (críticas + recomendadas) | Fase 2 + `odoo-effort-estimator` | valorado |
| 5 | **Preguntas para consultoría** (hallazgos no críticos + observaciones) | Fase 2/4 | cualitativo |
| 6 | **Propuesta — cómo seguir + oportunidades Indaws Core** (valorado) | Fase 4 + `odoo-effort-estimator` | valorado |

> **Factor IA.** Las horas que respaldan las secciones 4 y 6 son el esfuerzo real con factor IA
> aplicado — delega en la skill `odoo-effort-estimator`. Si vienen de una estimación técnica previa,
> ya lo llevan: no aplicarlo dos veces.

**Referencia cruzada obligatoria (A→B):** en la **tabla de salud 0-10** (sección 3), cuando se
genera el anexo, cada módulo con salud media/baja lleva la coletilla *"detalle y referencias de
código en el anexo técnico"*; y una nota al inicio de la sección 3 remite al Documento B. En la
valoración 0-10, una frase que avise "para más info, ver el anexo técnico".

### Encuadre del proyecto (Fase 0: `{en_produccion}` sí/no)
- **En producción** → sección 2 = "Estado de la base de datos" (volumen real de tablas, `db_state.sh`).
- **Pre-arranque / implantación en curso** (aún no operan; migración de datos pendiente) → sección 2 =
  **"Composición del desarrollo a medida"** (NO hablar de "BD vacía" ni conteos como si fuera producción);
  los riesgos se plantean como *"se materializarán al arrancar con datos reales → resolver antes del go-live"*.

### Sección 6 — propuesta value-based + oportunidades Core
Value-based (solo importes, nunca horas). **Dos opciones** (migrar = SIEMPRE Opción 2):
- **Opción 1 — Estabilizar y mantener**: arreglar críticos (€), traspaso a cloud (€; *deshoras=recargo*),
  Indaws Cloud (€/mes), mantenimiento (€/mes).
- **Opción 2 — Migrar** (solo si `{ofrecer_migracion}`=sí): + **formación** (N×2 h) + **soporte arranque**
  (1 mes = 10 % de la migración) + **migraciones recurrentes**: tabla aparte dentro de la Op.2 con el
  importe **por salto de versión, cada `cada_n_anios` años** = `{pct_migracion_recurrente}` de lo que
  costaría implantar hoy ese mismo sistema desde cero. Tres cosas van **en el cuerpo, no en nota al pie**:
  la **condición** (vale mientras no crezca la base instalada —custom e integraciones, **no usuarios**— más del umbral), que la cifra de referencia
  es una **reconstrucción del alcance actual y no una oferta**, y —si no hubo datos para calcularla— que
  es una **estimación aportada por el equipo**. Cálculo en `synthesis-and-crosscheck.md`.
- **Oportunidades Indaws Core (nuevo):** tabla que mapea cada módulo custom problemático →
  **paquete Indaws Core** que lo sustituye, **con precio de catálogo**. Ver Fase 4 y
  `synthesis-and-crosscheck.md`. Incluir aquí las herramientas de import/migración custom →
  **Framework de Migración e Integración de Datos** (import/export Indaws Core).

---

## Documento B — Anexo técnico

Autónomo, para el equipo técnico del cliente. Estructura:

| # | Apartado | Origen |
|---|----------|--------|
| B1 | **Detalle módulo por módulo** — por cada custom: *Qué hace* + *Errores encontrados* + salud | Fase 2 (`odoo-code-auditor`) |
| B2 | **Análisis de flujos de negocio** — trazado de los flujos clave (venta→fabricación, etc.) y dónde se rompen | Fase 2b (flujos) |
| B3 | **Bugs de runtime latentes** — **Código actual (fragmento) + Reproducción + Fix**, con `file:line` | Fase 2b |
| B4 | **Rendimiento / corrupción / inconsistencia** — mismo formato Código+Problema+Impacto+Fix | Fase 2b |
| B5 | **Matriz de seguridad** — ACL/grupos/record rules/`sudo`/controllers por módulo + hallazgos por gravedad | Fase 2b |
| B6 | **Inventario técnico** — modelos nuevos/extendidos, campos, vistas, wizards, crons, JS por módulo | Fase 1/2b |
| B7 | **Overrides de core y acoplamiento** — computes sin `super()`, punto caliente por modelo, monkey-patches JS globales, grafo de dependencias | Fase 2b |
| B8 | **Dependencias Python** — versión/estado (descontinuada/de pago/no portable) y si están declaradas | Fase 2b |
| B9 | **Estado de tests** — cobertura actual + plan mínimo | Fase 2b |

- Plantilla: `assets/anexo_tecnico_template.md`.
- **Sí lleva `file:line`, nombres de método/error y fragmentos de código** — es su razón de ser. El ejecutivo NO.
- Los bugs/flujos que estallan en producción se **trazan al código**; distinguir hecho vs. hipótesis.
- **Adaptar la profundidad al público:** si el técnico del cliente es de nivel alto, B3-B8 con `file:line`
  y fragmentos son lo que hace que el informe se tome en serio.

## Maquetación (ambos documentos)
- **Cada apartado empieza en página nueva.** CSS: `.pad>.panel{break-before:page;}` +
  `.pad>.panel:first-child{break-before:avoid;}`.
- **NO usar tablas de 4+ columnas con `<code>` largos** (desbordan la página en A4) → usar **bloques
  verticales** por elemento. Aplicar `word-break:break-word;overflow-wrap:anywhere` a los `<code>`.
- Notas/salvedades → **asterisco + `.note`** de sección (ver `dynapps-brand/references/pdf-reports.md`).
- **Verificación visual obligatoria:** además de `verify.py`, renderizar a PNG las páginas con tablas y
  confirmar que **nada se corta por el borde derecho** antes de entregar.

## Prohibido en AMBOS documentos
- **Cualquier mención a otro partner/proveedor** (redactar en neutro: "la instalación", "el sistema").
- **Horas / esfuerzo / tarifa €/h.** Las secciones valoradas muestran **solo importe** (€). El
  desglose de horas es interno (para `quotation-builder`). *(Nota: el `.md` interno de trabajo sí puede
  llevar horas para dimensionar; el PDF entregable, nunca.)*

## Tono y formato
- Cliente-facing pero técnico-honesto: datos duros, cada afirmación respaldada.
- Separar **hecho** (código/BD) de **recomendación/hipótesis**.
- El resumen ejecutivo (sección 1) se escribe **al final**, destilando el resto.
- PDF final con `common-skills:pdf-renderer` (Playwright), nunca wkhtmltopdf.

## Salida
Carpeta **`docs/`** desde el cwd (o ruta indicada):
```
docs/
├── rescue_pack_<cliente>.md                 # informe ejecutivo (assets/report_template.md)
├── rescue_pack_<cliente>.pdf                # Documento A (marca)
├── rescue_pack_<cliente>_anexo_tecnico.md   # anexo técnico (si {detalle}=completo)
├── rescue_pack_<cliente>_anexo_tecnico.pdf  # Documento B (marca)
├── studio_extract/                          # INVENTORY.md + json/ + code/server_actions/
└── odoo_studio/                             # módulo consolidado (si se generó en Fase 3b)
```

## Paso de cierre: PDF con marca
Tras validar los `.md`: identidad con **`dynapps-brand`** (portada, paleta, tono) + render con
**`pdf-renderer`** (Playwright/Chromium). Los dos PDF comparten identidad; el anexo lleva "Anexo
técnico" en el pie para distinguirlo del ejecutivo.
