# Checklist y Core Compliance Score

Evalua cada bloque, anota hallazgos con `file:line`, asigna severidad y calcula
el score. Filtra siempre con `gotchas.md` antes de contar un fallo.

## Severidades

| Severidad | Significado | Efecto en score |
|-----------|-------------|-----------------|
| 🔴 Critico | Rompe la reutilizacion multi-cliente o reinventa el estandar | Penaliza fuerte; bloquea "apto Core" |
| 🟠 Alto | Compromete filosofia pero acotable | Penaliza |
| 🟡 Medio | Empaquetado/documentacion/forma | Penaliza leve |
| 🔵 Info | Mejora recomendable | No penaliza |

## Bloques y pesos (Core Compliance Score, 0-100)

| Bloque | Peso | Que se evalua |
|--------|------|---------------|
| A. Multi-cliente | 35 | Anti-patrones 1,2,3,4,6 (hardcoding, logica por cliente, datos cliente, parametrizacion, acoplamiento) |
| B. Anti-codigo / cercania al estandar | 25 | Anti-patron 5 + veredicto de classification-guide y code-auditor (Standard Features, Golden Rules, Core Supremacy). "Extiende, no reinventa" se valida contra la superficie publica REAL del padre (paso 3.5), no contra docstrings; si el padre no esta disponible, marcar B como "inferido, no verificado" |
| C. Naming y empaquetado | 15 | Anti-patron 7 (nombre `indaws_*`, paquete, manifest, no `x_`). `version` en su serie vigente es correcto; solo penaliza si el usuario pidio migracion y aun no esta en la serie target |
| D. Documentacion | 10 | Anti-patron 8 (README/manifest util DENTRO del modulo; la ficha en Conocimiento NO se audita) |
| E. Calidad (delegado) | 15 | Sintesis de odoo-code-auditor: Conventions, Security, Performance, Code Quality (+ Migrability **solo si hay migracion pedida**) |

### Calculo

Cada bloque empieza en su peso maximo. Resta:
- 🔴 Critico: -40% del peso del bloque por hallazgo (min 0).
- 🟠 Alto: -20% del peso del bloque.
- 🟡 Medio: -10% del peso del bloque.
- 🔵 Info: 0.

`Core Compliance Score = suma de bloques` (0-100).

Para el bloque E, normaliza el resultado de odoo-code-auditor (su score /10 *
15) y resta criticos de seguridad como 🔴.

### Veredicto

| Score | Veredicto |
|-------|-----------|
| ≥ 85 y 0 criticos | ✅ Apto como Indaws Core |
| 70–84 y 0 criticos | 🟠 Apto con reservas — resolver Altos antes del PR |
| < 70 o ≥1 critico | 🔴 No apto como Core — requiere refactor |

Un solo 🔴 en el bloque A (hardcoding de cliente) o en B (reinventa estandar /
viola Golden Rule) baja el veredicto a "No apto" aunque el score sea alto: son
incumplimientos de la esencia de Core.

## Checklist operativo

### A. Multi-cliente
- [ ] Sin identificadores fiscales / razon social de cliente en codigo.
- [ ] Sin IDs de BD hardcodeados (`browse(n)`, domains con ids, `default=n`).
- [ ] Sin emails / URLs / IPs / rutas absolutas de cliente embebidos.
- [ ] Sin ramas `if cliente == ...` / logica por nombre de compania.
- [ ] `data/` sin registros propios de un cliente con `noupdate`.
- [ ] Lo que varia por cliente esta parametrizado (`res.config.settings`,
      `ir.config_parameter`, campos en `res.company`).
- [ ] Multi-compania respetado (`company_id`, reglas de registro) cuando aplica.
- [ ] `depends` sin modulos custom de un cliente.
- [ ] **Override de comportamiento nativo (create/write/constrains/checkout/
      post_init/ir.rule) tras flag opt-in default OFF + hook real overridable**
      (anti-patron 9). Sin flag → 🔴. Config funcional neutra ya existente
      (enum, `groups=`) cuenta como gate valido.
- [ ] `ir.rule` nuevas atadas a grupo, no globales con fallback permisivo
      (anti-patron 10).

### B. Anti-codigo / cercania al estandar
- [ ] No reimplementa funcionalidad nativa de Odoo (veredicto code-auditor
      Standard Features).
- [ ] No viola GR1 (import/export → `indaws_data_*`).
- [ ] No viola GR2 (dashboards/KPIs → Spreadsheets).
- [ ] No reinventa funcionalidad de un modulo Core existente (Core Supremacy,
      via classification-guide).
- [ ] El alcance es "solucion configurable", no "el flujo del cliente X".

### C. Naming y empaquetado
- [ ] Nombre tecnico `indaws_*`.
- [ ] Encaja en un paquete por area (no huerfano).
- [ ] Manifest: `author` Indaws, `license`, `version` formato Odoo
      (`<serie>.x.y.z`), `category` coherente, `website`.
- [ ] Sin `x_`/Studio dentro del modulo.
- [ ] **Modulo nuevo**: version `<serie>.1.0.0` (no versiones de desarrollo
      avanzadas) y SIN carpetas `migrations/<serie>.1.x.x/` de saltos dev
      (anti-patron 11).
- [ ] `i18n/` solo `es.po` (sin `es_ES.po` ni `*.pot`) — convencion del repo.
- [ ] `auto_install: True` solo si es infraestructura (no feature opt-in).
- [ ] `icon.png` / URL de logo = canonico del repo (salvo rebranding global en
      curso → ver `gotchas.md`).

### D. Documentacion
- [ ] README o `static/description/index.html` presente y util.
- [ ] Manifest con `summary`/`description` informativos.
- [ ] (La ficha en Conocimiento NO se audita: proceso interno del equipo,
      fuera del alcance de este auditor.)

### E. Calidad (delegado)
- [ ] Version derivada del `__manifest__.py`; target de migracion solo si el
      usuario lo pidio (si no, sin migracion).
- [ ] odoo-code-auditor ejecutado con Standard Features + Conventions +
      Security + Performance + Code Quality (+ Migrability solo si hay
      migracion pedida, indicando el target).
- [ ] Criticos de seguridad incorporados como 🔴.
- [ ] Score normalizado al bloque E.
