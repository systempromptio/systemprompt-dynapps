# Capas de enforcement (persisten entre sesiones)

La skill `migration-ticket` solo aplica **cuando se invoca**. Estas dos capas, en
cambio, **sobreviven a una sesión nueva y a reiniciar el ordenador** sin que nadie
se acuerde de nada, y **referencian los mismos IDs** de `conventions.md` (por eso
no hay desconexión):

1. **CLAUDE.md del repo de migración** — se carga automáticamente en cada sesión.
2. **Hooks de `settings.json`** — los fuerza el harness en cada acción (no se
   olvida ni se salta).

(La tercera capa durable, la **memoria**, ya la mantiene la Fase 5 de la skill.)

---

## 1. Bloque para el CLAUDE.md del repo de migración

Pegar en el `CLAUDE.md` del buildout/repo de migración (o en el global si se
trabaja siempre en migración). Es un **puntero**, no una copia de las reglas:

```markdown
## Flujo de migración (Dynapps Bélgica)

Para CUALQUIER cambio de un ticket de migración, ejecuta la skill
`migration-ticket` y sigue su Definition of Done. Convenciones DURAS (registro
completo en la skill → `references/conventions.md`):

- Código y comentarios SIEMPRE en inglés [LANG-EN]. Imita el estilo del fichero [STYLE].
- Rama `feature/<TICKET>`; commits SIN `Co-Authored-By`; NUNCA `push --force` en
  rama con MR (corregir con commits nuevos) [GIT-BRANCH, GIT-NOCOAUTHOR, GIT-NOFORCE].
- MR no se mergea sin pipeline VERDE (checklog-odoo marca warnings); si el CI no
  corre por accesos, gate = verde en local justificado [CI-GREEN, CI-EXTERNAL].
- Verifica en local lo verificable antes de subir [VERIFY-LOCAL].
- Un fix NO puede perder funcionalidad de forma indirecta: corre `odoo-migration-parity`
  si rozas vistas/campos/datos [ANTI-REGRESSION].
- Fix de datos = par pre-upload SQL + post-migration [DATA-PREUPLOAD-POSTMIG].

Esto es el flujo BÉLGICA; NO uses `git-manager`/`ticket-solver` (flujo España).
```

---

## 2. Hooks de `settings.json`

Fuerzan localmente las reglas **mecanizables**. Aplicar con la skill `update-config`
(conoce el esquema vigente de hooks). Cada hook indica el **ID** que hace cumplir.

> El esquema exacto de `settings.json` lo fija `update-config`; aquí se define la
> intención (matcher + condición + acción + ID). No editar `settings.json` a mano
> sin pasar por esa skill.

### H1 — bloquear `Co-Authored-By` en commits [GIT-NOCOAUTHOR]
- **Evento/matcher**: `PreToolUse` sobre `Bash`.
- **Condición**: el comando es un `git commit` cuyo mensaje contiene `Co-Authored-By`.
- **Acción**: **bloquear** (exit no-cero) con mensaje: *"GIT-NOCOAUTHOR: quita la
  línea Co-Authored-By (convención de migración Bélgica)."*

### H2 — bloquear force-push [GIT-NOFORCE]
- **Evento/matcher**: `PreToolUse` sobre `Bash`.
- **Condición**: el comando es `git push` con `--force` / `-f` / `--force-with-lease`.
- **Acción**: **bloquear** con mensaje: *"GIT-NOFORCE: no reescribas historia en
  una rama con MR; corrige con commits nuevos encima."*

### H3 — avisar de comentarios en español [LANG-EN] (opcional, warning)
- **Evento/matcher**: `PreToolUse` sobre `Bash` en `git commit` (o `PostToolUse`
  sobre `Edit`/`Write` de ficheros de código).
- **Condición** (heurística): líneas **añadidas** de comentario que contienen
  palabras claramente españolas (`# ...` / `// ...` / `<!-- ... -->` con
  `de la|para|según|añad|elimin|vac|correo|actualiz|comprob`).
- **Acción**: **avisar** (no bloquear; heurística con falsos positivos) apuntando a
  LANG-EN. El bloqueo duro de idioma se deja al review y a la skill.

Referencia de implementación (bash, se cablea vía `update-config`):

```bash
# H2 example: PreToolUse/Bash guard. The hook receives the tool input as JSON on
# stdin; block when a git push uses a force flag. Comments in English (LANG-EN).
cmd="$(jq -r '.tool_input.command // empty')"
case "$cmd" in
  *"git push"*"--force"*|*"git push"*" -f"*|*"git push"*"--force-with-lease"*)
    echo "GIT-NOFORCE: force-push blocked on MR branches; add new commits instead." >&2
    exit 2 ;;
esac
```

> Cobertura honesta: **H1/H2 se pueden forzar** (inspeccionan el comando). **CI-GREEN**
> lo fuerza el pipeline de GitLab (no un hook local). **ANTI-REGRESSION**,
> **VERIFY-LOCAL**, **DATA-PREUPLOAD-POSTMIG** son de criterio → los cubre la skill,
> no un hook. Cada regla se aplica donde de verdad se puede aplicar; el registro
> (`conventions.md`) dice con qué mecanismo, para que no quede ninguna suelta.
