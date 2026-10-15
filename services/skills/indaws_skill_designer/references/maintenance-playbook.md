# Maintenance Playbook — mantener el ecosistema ágil

160 skills × ~1.200 references es mucha superficie. La deuda se controla con
**4 principios** + **1 ritual por release** + **tooling que atrapa lo estructural solo**.

## Principios (reducen lo que hay que tocar)

1. **SSOT de lo volátil = `odoo-version-api-reference`.** Las firmas y diffs
   **por versión** viven SOLO ahí (`v18-base/` + `diffs/vN-diff.md`). El resto de
   skills llevan **conceptos/patrones** (estables) y **delegan** lo version-pinned.
   → una versión nueva de Odoo = actualizar **un** fichero, no 25.
2. **Estable vs volátil explícito.** SKILL.md + references de concepto = version-stable
   (no se tocan por release). Lo version-pinned va marcado o delegado. No mezclar.
3. **Procedencia = recibo best-effort.** Las citas `fichero:línea` documentan dónde
   se verificó un hecho; la línea **driftea** y no pasa nada. El **hecho** es lo que
   vale. Solo re-verificar cuando se usa y huele mal. NO se persigue cada nº de línea.
4. **Podar por uso.** Mantener lo que se dispara y ayuda; jubilar lo que nadie usa.
   Menos skills = menos superficie. Consolidar si dos skills siempre se usan juntas.

## Ritual por major de Odoo (la única tarea humana recurrente, ~1 día)

1. `odoo-version-api-reference`: añadir `diffs/v{N+1}-diff.md` (una pasada de diff
   contra fuente; ver `references/shared/odoo-source.md` para resolver la fuente).
2. Actualizar la **chuleta de diffs** del CLAUDE.md global (subset always-loaded).
3. `staleness.py --latest {N}` → lista A = skills que codifican facts de v{N};
   revisarlas para v{N+1} (solo esas, no las 160).
4. `doctor.py` → confirmar **0 regresiones** (issues nuevos vs baseline). Si los
   issues restantes son aceptados (excepciones conocidas), `doctor.py --update-baseline`.

> Los scripts **auto-detectan la raíz del ecosistema** desde su ubicación (o
> `$SKILLS_ROOT`). Si no la resuelven, **te la piden** — pásala como argumento.
> No hay rutas locales hardcodeadas.

## Tooling (lo estructural se mantiene solo)

| Script | Qué | Cuándo |
|--------|-----|--------|
| `scripts/doctor.py` | Bundle: verify_suite + duplicados + validate + catalog-freshness, **vs baseline** → reporta solo REGRESIONES. `--update-baseline` para aceptar el estado actual | pre-push / CI / antes de release |
| `scripts/staleness.py --latest N` | Lista focalizada de re-revisión por versión | en cada major nuevo |
| `scripts/verify_suite.py` | 12 checks por skill (descripción >1024 = FAIL, >600 = WARN; + WARN trigger-lint si la description no orienta el disparo) | al editar una skill |
| `scripts/generate_catalog.py` | Regenera CATALOG.md (doctor avisa `stale-catalog` si está desfasado) | tras add/move/delete |
| `scripts/detect_duplicates.py` / `validate_ecosystem.py` | Duplicados / refs rotas / simetría / orphan-refs | gobernanza |
| `scripts/install-hooks.sh` | Instala el pre-push hook (doctor) en uno/todos los repos de suite | una vez por repo |
| `scripts/ci/doctor.yml` | Plantilla GitHub Actions (checkout de suites hermanas + doctor) | copiar a `.github/workflows/` |

**Gates por convención (extension point).** `doctor.py` autodescubre por glob `*/skills/*/scripts/{check_registry,gen_index,check_xrefs}.py` y los ejecuta — una skill/suite puede traer su **propio gate** y el doctor lo recoge sin acoplarse. Patrón: deja un `check_*.py` que imprima `N error(es)` y salga ≠0; sus errores entran al set vs baseline como el resto. (Ej. real: deep-knowledge añadió registro SSOT + índice anti-drift + cross-refs colgantes así.)

### Baseline (la señal es la REGRESIÓN, no el rojo permanente)
`doctor.py` compara los issues actuales contra `.doctor-baseline` (excepciones
aceptadas: duplicados consentidos, WIP, etc.) y **solo falla si hay issues NUEVOS**.
Así el verde significa "no he empeorado nada", no "todo perfecto". Cuando una
excepción es intencionada → `doctor.py --update-baseline` la incorpora. Cuando un
issue del baseline se resuelve, doctor lo lista como "resuelto" para que limpies.

### Enforcement (atrapa deuda antes de que entre)
`scripts/install-hooks.sh [--all <root>]` instala un **pre-push** que corre
`doctor.py` sobre la raíz (auto-detectada o `$SKILLS_ROOT`) y bloquea si hay
regresiones. Para CI, copia `scripts/ci/doctor.yml` a `.github/workflows/` y rellena
`ORG`/`SUITES` (el doctor necesita todas las suites hermanas, por eso las clona).

## Convenciones aplicadas por el tooling

1. **Integridad de referencias** (automática): `validate_ecosystem`/`doctor` fallan
   si un SKILL.md apunta a un `references/[algo].md` o `common:references/shared/[algo].md`
   que no existe. → enlaces rotos se atrapan en pre-commit, no en producción.
   - **Antes de editar un SSOT compartido**, corre `scripts/blast_radius.py <ref>` para ver
     QUÉ skills lo heredan (sin arg = todos los shared refs + nº de consumidores; un cambio
     en p.ej. `odoo-connection.md` afecta a ~85 skills). Nunca edites un SSOT a ciegas.
   - **Colisión de triggers** (medida, no a ojo): `scripts/trigger_collisions.py` lista la
     misma frase-trigger en ≥2 skills (HARD) y pares con alto solape de keywords (SOFT,
     Jaccard). Corre tras tocar descripciones/triggers. Algunas colisiones son intencionadas
     (hub + atajos, p.ej. `git-manager`/`git-new-repo`); el eval te dice cuáles revisar.
2. **Marcador de verificación** (precisión de staleness): en una reference
   source-verified, añade arriba un comentario HTML (invisible al render):
   `<!-- verified: v18.0 @ 2026-06 -->`. `staleness.py` lo lista (sección C) → sabes
   exactamente qué se verificó contra qué versión y cuándo. Se añade al verificar,
   no en bloque.
3. **Deprecación** (podar sin romper): marca la skill con frontmatter
   `status: deprecated` (campo reconocido, no es ghost FM) y apunta al reemplazo en
   el cuerpo. `validate_ecosystem` avisa `[DEPRECATED_CONSUMED]` si alguien aún la
   referencia → migras consumidores antes de borrar.
4. **Simetría de delegación** (`[ASYMMETRIC_DEP]`, warning): si A declara `Delegates
   to` B, B debería listar A en `Consumed by`. Los SSOT de alto fan-in (version-api,
   pilot, deep-knowledge…) están exentos: no enumeran consumidores. Es advisory —
   revísalo, no bloquea.
5. **Orphan refs** (`[ORPHAN_REF]`, warning): un `.md` bajo `references/` que NINGÚN
   SKILL.md enlaza → o lo enlazas o lo borras. Advisory.
6. **Trigger-lint** (warning en `verify_suite`): la `description` debe llevar lenguaje
   de disparo (Triggers/Use-when/Invocar/"cuando") — es el selector entre 160 skills.

## IDEAS — canal de ideas del equipo (no es doc de skill)

Cola de temas a revisar con el equipo (propuestas, decisiones pendientes que
afectan a skills). Es **comunicación de equipo, no documentación de una skill** —
por eso NO vive dentro de la carpeta de la skill (ver anti-pattern #22).

- **Ubicación:** un `IDEAS.md` por suite, en la **raíz del repo de la suite**
  (`<root>/ai-claude-<suite>/IDEAS.md`). Se crea bajo demanda (cuando esa suite
  tiene su primer tema), no se pre-crean los 9.
- **Nunca enlazarlo desde un SKILL.md.** Si entra en REFERENCES, el sistema de
  progressive-disclosure lo auto-lee en CADA trigger → tokens del backlog en cada
  invocación. Al no enlazarlo, cuesta 0 y solo se lee al gestionar skills.
- **Formato:** `- [ ] [<skill>] <idea> — <por qué> (añadido YYYY-MM-DD)`, con
  `(bloqueado por: …)` cuando dependa de algo externo. Cerrados → bajo `## Resuelto`.
- **Vista agregada:** `scripts/todos.py [root] [--all]` junta todos los `IDEAS.md`
  de todas las suites en un único backlog (ordenado, con flag BLOQUEADO y fecha).
- **Tres contenedores distintos, no mezclar:** `IDEAS.md` (canal de equipo) ·
  `docs/` de la skill (sus planes técnicos multi-fase propios) · `references/`
  (contenido que la skill necesita para funcionar).

## Regla de oro
Antes de **añadir** una skill nueva, pregunta: ¿es un concepto estable nuevo, o
son facts volátiles que deberían vivir en `odoo-version-api-reference`? Si es lo
segundo, NO crees skill: amplía la SSOT.
