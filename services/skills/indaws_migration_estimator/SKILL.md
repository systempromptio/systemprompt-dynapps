---
name: indaws-migration-estimator
version: 0.2.0
effort: high
model: opus
description: >-
  Estima y presupuesta migraciones Odoo que Indaws ejecuta para Dynapps:
  analiza módulos/BD con reglas fijas Dynapps (1-a-1, 45 EUR/h, 100-150h
  rango sano), genera informe PDF de marca y crea oferta draft en Odoo. No
  sustituye a odoo-database-analyzer para otros clientes. Triggers: "estimar
  migracion", "presupuesto de migracion", "migration estimate", "presupuestar
  migracion Dynapps", "informe de migracion para Janik", "cuantas horas esta
  migracion", "oferta migracion Dynapps".
allowed-tools: [Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion, Skill, Agent]
---

# Migration Estimator — Presupuestación de migraciones Dynapps

Estima una migración que Indaws hace **para Dynapps**, aplica las reglas fijas de
negocio, **calibra la sobreestimación** del motor, y entrega informe PDF +
presupuesto. Hace pocas preguntas y va directo al entregable.

> Contexto, reglas y flujo completo del suite en `migration-lab:DESIGN.md`.

Conexión a Odoo y límites: ver `references/shared/odoo-connection.md` y
`references/shared/batch-limits.md`.

## Inputs (pregunta solo lo imprescindible)
1. **Ruta al código** — carpeta/ZIP de módulos custom (descargados del Drive de Dynapps).
2. **BD** — según lo que haya disponible (en este orden de preferencia):
   - ruta a un **dump** (`.dump`/`.zip` de Odoo) **o** fichero **`.sql`/`.sql.gz`**, si se dispone; o
   - solo el **tamaño aprox.** en texto (ej. "15 GB") si no hay fichero.
   Pregunta "¿tienes dump/sql o solo el tamaño?" y acepta lo que el usuario dé. El
   fichero NO se restaura ni se conecta aquí (ver GOTCHA 3); solo se usa su **tamaño
   en disco** (`ls -lh` / `du -h`) para dimensionar el bucket 2.
3. **Dónde generar el informe** — ruta de salida del PDF. Default: carpeta de trabajo
   del proyecto (junto al código). Confírmala si no se indica.
4. **Versión origen** (auto-deducir del repo/manifiestos si no se da; rango 14-19).
5. **Versión destino** (default **19.0**).
6. **Nombre del proyecto/cliente** (para descripción de la línea de presupuesto).

## Workflow

1. **Preparar** — Si el código es ZIP, descomprimir a carpeta de trabajo. Deducir
   versión origen de los `__manifest__.py` si no se indicó. Si para la BD se dio un
   fichero (dump/`.sql`), leer su **tamaño en disco** (`ls -lh`/`du -h`); si solo se
   dio el tamaño en texto, usarlo tal cual. Fijar la **ruta de salida del informe**
   (default: carpeta de trabajo del proyecto).
2. **Analizar** [delega] — Invoca la skill `odoo-database-analyzer` con el **perfil
   migration-lab**: tipo **1-a-1**, **sin testing funcional**, **100% IA**. Ver
   `references/business-rules.md` §Perfil-analyzer para los parámetros exactos.
3. **Calibrar en 4 buckets** [opus] — Aplica `references/calibration.md`:
   - **1 Code Migration** (variable): excluir OCA y nativos, eliminar validación
     humana, asignar horas/módulo por complejidad, **cap ~20h/módulo**.
   - **2 Database Upgrade** (variable): nº iteraciones × tamaño; **floor 20h**.
   - **3 Unit Testing** = **10h** por defecto (~2h si módulo único pequeño).
   - **4 Project Management** = **10h** por defecto.
   > **Buckets 3 y 4 son fijos a 10h por defecto.** Solo se recalculan si el proyecto
   > es **muy grande** (bucket 1 alto, muchos módulos/conectores o total que se acerca
   > al techo de 200h): en ese caso **pregunta al usuario** si quiere recalcularlos
   > (más cobertura de tests / más coordinación) antes de subirlos. No los infles por
   > defecto.
4. **Validar con la referencia paramétrica** [interno] — Contrasta el total contra el
   modelo LOC + base por tamaño de BD con `scripts/reference_check.py` (ver
   `references/validation.md`): `OK` (±15 %) sigue; `ABOVE` revisa inflado; `BELOW` revisa
   infraestimación. Informa del desvío **en el chat** y guarda `docs/estimate_check.json`.
   **Nunca** lo menciones en el PDF ni en el `sale.order`.
5. **Validar rango** — Total = 1+2+3+4. **100-150h** es la **media histórica** de
   estos proyectos, no un techo: úsala como referencia para detectar inflado del
   analyzer. Si **>200h**, no lo recortes a la fuerza: trátalo como **caso excepcional
   por volumen** (muchos módulos/conectores, saltos grandes) que **debe justificarse**
   en la sección 04 del informe. Solo recorta si el exceso viene de sobreestimación,
   no de volumen real.
6. **Desglose** — Tablas por bucket (ver estructura en `references/report.md`),
   total y € (×45).
7. **Informe PDF** [llama a `dynapps-brand`] — `dynapps-brand` es el SSOT del PDF:
   copia su plantilla `assets/templates/migration-report-template.html`, rellena las
   **4 secciones** (01 Executive Summary · 02 Code Migration · 03 Database/Testing/Mgmt ·
   04 Critical Points, estructura en `references/report.md`), renderiza con sus
   `scripts/gen.js` + `scripts/stamp.py` y **pasa el QA `scripts/verify.py`** (`RESULT: OK`: portada, **márgenes laterales** y
   hairlines). Si falla por márgenes, el contenido está desplazado a la derecha: parte el
   `code`/texto largo o revisa la celda/tarjeta que se ensancha (ver `report.md` §Render).
   NO clones el CSS ni los scripts aquí. **Solo horas, nunca € en el PDF.** Destinatario: **Janik**.
8. **Presupuesto** [delega] — Crea el `sale.order` con la skill `odoo-pilot`
   (ver `references/quotation.md`): borrador, línea "migración", qty=horas,
   precio 45, contacto Dynapps, adjunta el PDF. **No confirmar. No enviar correo.**
   **Persiste su referencia** en `docs/quotation.json` del proyecto para poder
   editarlo después. Si ya existe y el pedido sigue en borrador → **actualiza** ese
   (no dupliques); si está confirmado, pregunta antes (ver `quotation.md` §Persistir).

## GOTCHAS
1. **El analyzer SIEMPRE infla** (300-800h). Calibración y topes por módulo son
   guardarraíles **duros** contra ese inflado. Pero el rango total **100-150h es una
   media histórica, no un límite**: si tras calibrar bien el total supera el rango por
   **volumen real** (proyecto excepcional), es legítimo — recorta solo la
   sobreestimación, no las horas reales.
2. **NUNCA enviar correo desde Odoo.** Se crea el presupuesto y se deja listo; el
   envío a Janik es manual y fuera de Odoo, aunque el usuario diga "mándaselo".
3. **BD: tamaño o fichero, nunca restaurar.** Da igual que el input sea un dump/`.sql`
   o solo el tamaño en texto: **NO restaures ni conectes** a la BD. Si hay fichero,
   úsalo solo para leer su **tamaño en disco** (`ls -lh`/`du -h`) y dimensionar el bucket 2.
4. **Testing funcional = NO.** Lo hace Dynapps. Si el analyzer mete horas de
   validación funcional o de "confirmación humana", elimínalas.
5. **Tipo de migración = 1-a-1 siempre.** Dynapps migra todo tal cual; no propongas
   refactor/Studio aunque el analyzer lo sugiera.
6. **Con IA un módulo rara vez pasa de 20h.** Topar al revisar el desglose.
7. **OCA y terceros = mismo árbol de decisión** (ver `calibration.md` §Bucket 1.1):
   (1) hay port v19 → fuera; (2) Odoo lo absorbió al core y OCA lo descartó → usar
   nativo, fuera; (3) **no existe port ni core y hay que portarlo → se estima en el
   presupuesto Y se anota en el PDF (sección 04) como decisión** del cliente
   (portar o no). Regla de oro: todo lo que no existe en destino se cuantifica y se
   deja decidir, no se asume. Si existe en OCA **y** en Odoo (ej. comisiones) → mantener
   el instalado (fuera). "Drift": OCA/vendor parcheado localmente vuelve a scope.
8. **Estructura fija de 4 buckets / 4 secciones.** No inventar otra; seguir
   `references/report.md` (formato Geosquare/Luminex; el antiguo 1A/1B está deprecado).

## NEVER
- Enviar email desde Odoo ni confirmar el sale.order automáticamente.
- Mencionar la validación paramétrica (fórmula, LOC × tarifa, talla S/M/L, desvío) en el
  informe PDF o en el sale.order — es un control interno.
- **Incluir importes en € en el informe PDF** — solo horas; el precio (45 €/h) va solo en el sale.order.
- Incluir testing funcional ni horas de validación humana.
- Aceptar la estimación cruda del analyzer sin calibrar.
- Proponer tipo de migración distinto de 1-a-1.

## ALWAYS
- Pasar la validación paramétrica (`scripts/reference_check.py`) antes de generar
  entregables y reportar el desvío en el chat.
- Sumar los fijos 10h test + 10h PM por defecto (salvo módulo único pequeño → menos
  test). Recalcularlos solo si el proyecto es muy grande y tras preguntar al usuario.
- Avisar si el total supera 200h y justificarlo por volumen en la sección 04 (caso
  excepcional legítimo), antes de generar entregables.
- Usar 45 €/h y la marca Dynapps en el PDF.

## Delegates to
| Skill | Suite | For what |
|-------|-------|----------|
| `odoo-database-analyzer` | common | Motor de análisis de código y esfuerzo (perfil 1-a-1) |
| `odoo-module-scanner` | common | Inventario de módulos si hace falta antes del análisis |
| `dynapps-brand` | common | Estilo de marca del informe PDF (light-only, fuentes, logo) |
| `pdf-renderer` | common | Render HTML→PDF (Playwright) + reglas de paginación (SSOT) |
| `odoo-pilot` | common | Creación del sale.order (presupuesto) vía API |

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| `migration-portfolio` | migration-lab | Lanza estimaciones en lote sobre la cartera de migraciones |

## REFERENCES
- `references/business-rules.md` — Reglas fijas Dynapps + perfil del analyzer
- `references/calibration.md` — Cómo recortar la sobreestimación + topes
- `references/validation.md` — Validación paramétrica interna (LOC + base por tamaño de BD)
- `scripts/reference_check.py` — Cuenta LOC en scope y calcula referencia/desvío/veredicto
- `references/report.md` — Informe PDF (dynapps-brand + pdf-renderer); estructura 4 secciones, solo horas
- `references/quotation.md` — Creación del sale.order + regla NO-email
- `migration-lab:DESIGN.md` — Diseño del suite y contexto de negocio
