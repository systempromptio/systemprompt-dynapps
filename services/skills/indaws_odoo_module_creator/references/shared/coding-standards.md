# Coding Standards (shared)

Cross-cutting conventions for ANY skill that generates source code that may be committed to git. Single source of truth — referenced vertically by code-generating skills across suites. Do not duplicate this rule inside individual skills; link here.

> **Entrega**: qué se exige para dar una tarea por terminada (prueba de flujo ejecutable, registro en
> tarea, manual, consultar gotchas al abrir) vive en `common:references/shared/delivery-standard.md`
> (estándar "Desarrollador 2.0"). Este doc es el **cómo se escribe** el código; aquel, el **qué se entrega**.

> **Fuente Odoo:** si necesitas verificar contra el código fuente, resuelve la
> ruta vía `common:references/shared/odoo-source.md` — **no asumas `$ODOO_SRC`**.
> Las citas `fichero:línea` en las references son procedencia del autor, no la
> ruta del usuario.

## Source code is always written in English

All **source code is in English, regardless of the client, their country, or the language they speak.** This is non-negotiable and independent of the project's locale.

English applies to:
- Identifiers — variable, function, method, class, model and field **technical** names (`partner_invoice_count`, not `numero_facturas_cliente`)
- Comments and docstrings
- Commit messages, branch names, PR titles/descriptions
- Log messages and internal exception text
- File and module names
- XML record `id`s, action/menu technical names, external IDs

**Translated / localized (NOT forced to English):**
- User-facing strings: field `string=`, view labels, menu labels, button text, report content, help texts, selection labels
- These go through Odoo's i18n (`_()` + `.po` files), with the source term in English and translations per language

### Why
- The codebase outlives any single client engagement and is maintained by a rotating, multilingual team — English is the lingua franca that keeps it readable for everyone.
- Mixing languages in identifiers makes grep, refactors, and reviews unreliable.
- Odoo core, OCA and Indaws Core are all in English; matching them keeps overrides and inheritance coherent.

### Examples
```python
# Good — English code, Spanish-facing label via i18n
unpaid_invoice_count = fields.Integer(string="Facturas pendientes")

# Bad — Spanish identifier
numero_facturas_pendientes = fields.Integer(string="Facturas pendientes")
```

```python
# Good
def _compute_overdue_balance(self):
    """Sum of move lines past their due date."""

# Bad
def _calcular_saldo_vencido(self):
    """Suma de apuntes vencidos."""
```

## Enforcement
- **Generation skills** must follow this when emitting code and surface it as an `ALWAYS` rule.
- **Review skills** (`odoo-code-auditor`, `quality-gate`) flag any non-English identifier/comment/commit, even when the client requested otherwise — only user-facing strings are exempt. **Severity by origin:**
  - Code **generated or changed in the work under review** → blocking (FAIL). We control it; there's no excuse to ship non-English.
  - **Pre-existing legacy** code (not touched by this work) → advisory (WARN). Surface it, don't block the deploy/review over it — flag for gradual cleanup.

## Anti-laxity antipatterns — fix or justify (MANDATORY)

Estos antipatrones "funcionan en el camino feliz y rompen en el borde". El defecto
humano y del modelo es escribir el caso concreto que tenemos delante; estas reglas
invierten ese defecto. **Cualquier skill que genere o modifique código DEBE, antes de
entregar, auto-auditarse contra esta lista**, y para cada coincidencia: **(a) corregirla,
o (b) mantenerla SOLO con una justificación explícita inline** `# JUSTIFICADO: <razón>`
(en XML: `<!-- JUSTIFICADO: ... -->`). **Nunca entregar un antipatrón sin justificar.**
En el resumen final, listar los que se mantuvieron justificados.

| Antipatrón | Regla | Detección |
|---|---|---|
| **Hardcodeo de dato de cliente** (nombre/código/CIF/IBAN/cuenta) en módulo `indaws_*` o reutilizable | Va a config/dato (`res.company`, `dataflow.connection`, `ir.config_parameter`, dato XML), no literal en código | literal VAT/IBAN/código de empresa fuera de tests/fixtures |
| **Secreto literal** (`password`/`token`/`api_key = "..."`) | Va a la conexión/credencial segura, nunca en fuente | asignación con literal no vacío fuera de `fields.` y de tests |
| **Test hueco** (verde que no verifica nada) | Asserta el **valor exacto** y prueba que el input **llega al output**; no `assertTrue(existe)`/`len`/`in`/"no peta" como única aserción | `def test_*` sin `assert*` real |
| **Transliteración tras concatenar** (`to_ascii`/encode sobre la línea ya unida) en ancho fijo | Aplicar **por campo, antes de ajustar a longitud** — una sustitución que expande (`€`→`EUR`) desalinea todo el registro | `to_ascii(...join...)` |
| **Truncado silencioso de numérico/importe** | Un overflow de campo debe **lanzar**, no truncar (`truncate=False`); truncar corrompe el dato fiscal | `[:length]`/`truncate=True` en campo de importe |
| **`float` para dinero** donde el redondeo importa; recompute de impuesto por línea sumado en vez de `amount_tax` | Usar el redondeo de Odoo; cuadrar base/cuota/total al céntimo | aritmética monetaria con float |
| **Fuga multi-company** | `search`/dominio sobre modelo multi-company sin filtro de compañía al iterar por cuenta/empresa | dominio sin `company_id` en bucle por compañía |
| **`commit` en método de negocio** o API que commitea a mitad sin idempotencia | El commit va en cron/queue boundary; si una API commitea (`_create_records_from_attachments`), añadir guarda de idempotencia | `cr.commit()` fuera de migración/cron |
| **Data `noupdate="1"` mantenida como producto** que cambia | Los fixes a esos registros NO propagan por `-u`; añadir migración o `noupdate="0"` | cambio en data noupdate sin migración |
| **`@api.depends` incompleto** | Declarar toda dependencia que el compute lee | compute que lee un campo ausente de `@depends` |
| **`check_company=True` decorativo** | El check solo se ENFORCE si el modelo define `_check_company_auto = True`; sin él, un M2O `check_company` NO bloquea cross-company (queda de adorno) | M2O con `check_company` en modelo sin `_check_company_auto` |
| **Modelo heredado (`_inherit` + `_name` nuevo) sin re-declarar `ir.rule`/índices** | Es tabla separada: las `ir.rule` (multi-company) e índices del modelo base NO la cubren — re-declararlos en el nuevo modelo | nuevo `_name` que hereda un modelo con `company_id` y no trae su propia regla multi-company |
| **Config que escala con el menú, no con registros** (módulo de operador/feature añade `menuitem` raíz por instancia) | La config específica de operador se alcanza desde su ficha (smart buttons) o bajo un submenú técnico gateado por grupo; añadir un operador = **0 menús nuevos** | `menuitem` raíz nuevo por cada operador/agente |

**Reparto de enforcement** (igual que el resto de este doc):
- **Skills de generación** (`odoo-module-creator`, `odoo-test-generator`,
  `indaws-core-developer`, `odoo-server-action-creator`, …): auto-auditoría
  obligatoria antes de entregar; corregir o justificar; surface como regla `ALWAYS`.
  El generador de tests crea además los casos adversarios
  (`odoo-test-generator:references/adversarial-cases.md`).
- **Skills de revisión** (`odoo-code-auditor`, `indaws-core-auditor`, `quality-gate`):
  marcan estos antipatrones; severidad por origen (código de este trabajo → bloqueante;
  legacy no tocado → advisory). Detalle en
  `odoo-code-auditor:references/hollow-tests.md`.

> Nota de alcance: esto solo ata a quien **usa las skills**. Para atrapar también el
> código escrito a mano, instalar el hook de pre-entrega a nivel de **proyecto**
> (`.claude/settings.json` commiteado) — la enforcement de verdad universal vive en
> config compartida del repo, no en config personal.

## Marca de procedencia (Made with)

Toda skill que **genera** código commiteable estampa una marca al final de cada fichero que crea o
regenera: `Made with <skill> · v<version>` (envuelta en el comentario del tipo de fichero). Es la
señal de adopción de skills (`grep -r "Made with"`). Las skills de **revisión NO** estampan el
fichero — dejan su traza en el informe/PR. Regla completa (sintaxis por tipo, excepciones CSV/JSON,
idempotencia): **`common:references/shared/skill-provenance-marker.md`**. Surface como `ALWAYS` en
cada skill generadora.
