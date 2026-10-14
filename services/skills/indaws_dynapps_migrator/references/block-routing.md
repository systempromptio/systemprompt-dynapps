# Block Routing — Dynapps Migrator

Mapa de decisiones: qué skill invocar en cada bloque, con qué parámetros y bajo qué condición.
La numeración de STOPs es la canónica de **9** (ver `stop-points.md`). El track de código
(BLOQUE 3) y el go-live (BLOQUE 5) ramifican por `scenario` (ver `scenarios.md`).

---

## BLOQUE 0 — Detección de estado

```
scenario        = resolve_scenario()       # scenarios.md (host repo → copier → infra)
machine_ready   = check_prerequisites()
buildout_exists = check_buildout(cliente, version_destino)
workdir_active  = check_workdir(cliente)   # indaws-migrate playbook-session

IF workdir_active  → RESUME MODE (ver resume-mode.md)
IF NOT machine_ready → BLOQUE 1.A
IF NOT buildout_exists → BLOQUE 1.B
ELSE → BLOQUE 1.C (Docker) → BLOQUE 2
```

Tras BLOQUE 0: imprimir requerimientos del `scenario` (`requirements.md`) + 1 confirmación,
luego correr autónomo hasta el hito "Entorno local listo" (`autonomous-mode.md`).

---

## BLOQUE 1.A — Prerequisites de máquina

**Skill:** `developer-skills:dynapps-dev-bootstrap`
**Condición:** `machine_ready == False`

```
Usa developer-skills:dynapps-dev-bootstrap.
Cliente: <cliente>, versión destino: <version_destino>.
Instala automáticamente todo lo que falte (SSH, pre-commit, copier, Docker, Vault).
Al llegar al login Vault/OIDC → STOP ① (el usuario lo ejecuta: ! vault-login).
```

**Output:** `machine_ready: true` → continuar BLOQUE 1.B.

---

## BLOQUE 1.B — Buildout del cliente

**Skill:** `developer-skills:dynapps-local-env-setup`
**Condición:** `machine_ready == True AND buildout_exists == False`

```
Usa developer-skills:dynapps-local-env-setup.
Cliente: <cliente>, versión destino: <version_destino>.
Repo: <repo>   Escenario: <scenario>
  - onpremise_gitlab → repo GitLab Dynapps (flujo por defecto)
  - odoosh_github    → pasar --odoo-sh y responder "N" a la pregunta de GitLab
                       (repo git@github.com:DynAppsNV/<cliente>.git)
En modo autónomo, SALTAR los pasos de IDE (SmartGit/PyCharm): no hacen falta para arrancar.
```

**Output:**
- Buildout en `$ODOO_SRC/buildouts/<cliente><version_destino>/`
- PostgreSQL local verificado y activo

**Condición de salida:** buildout OK → continuar BLOQUE 1.C.

---

## BLOQUE 1.C — Docker para iterate loop

**Ejecutado por:** el orquestador directamente.
**Condición:** Siempre, antes del BLOQUE 2.

```bash
[ -f migration_kit_<cliente>/docker-compose.yml ] || cat > migration_kit_<cliente>/docker-compose.yml << 'EOF'
version: "3.8"
services:
  db:
    image: postgres:15
    environment:
      POSTGRES_USER: odoo
      POSTGRES_PASSWORD: odoo
      POSTGRES_DB: postgres
    ports:
      - "5433:5432"
    volumes:
      - pgdata:/var/lib/postgresql/data
    restart: unless-stopped
volumes:
  pgdata:
EOF
cd migration_kit_<cliente> && docker compose up -d db
docker compose exec db pg_isready -U odoo
```

Ver `dynapps-code-migration/references/docker-iterate-env.md`.

---

## BLOQUE 2 — Upgrade de BD

### 2.A — Upgrade Service

**Skill:** `developer-skills:dynapps-db-upgrade-runner`
**Condición:** `scenario == onpremise_gitlab` (path on-premise: SSH a prod).

```
Usa developer-skills:dynapps-db-upgrade-runner.
Cliente: <cliente>, v<version_origen> → v<version_destino>.
Infraestructura: <infraestructura>.
Lanzar el upgrade en background + vigilante; avanzar en paralelo lo que no depende del dump.
Decisión test/producción → STOP (irreducible). Por defecto: test.
GATE de neutralización (duro): verificar is_neutralized + mail/cron/EDI antes de odoo-bin.
```

> Para `scenario == odoosh_github`: el upgrade es **nativo de odoo.sh** (siempre `test`),
> gestionado por `odoo-database-upgrader` FASE 0 con `target=odoo.sh`. No hay path SSH a prod.

**Output:** dump migrado disponible en local + BD restaurada + **neutralizada** en PostgreSQL.

### 2.B — Planificación + iterate loop

**Skill:** `developer-skills:odoo-database-upgrader`

```
Usa developer-skills:odoo-database-upgrader para las FASES 0-5.
target_deployment = (odoosh_github → odoo.sh | onpremise_gitlab → indaws/custom)

FASE 0: indaws-migrate init → workdir + repo odoo-migkit-<cliente> en GitHub
FASE 1: pg_restore dump en contenedor Docker (5433) — dump inmutable fuera del workdir
FASE 2: STOP ③ — esperar estudio previo del consultor (md/xlsx/pdf); si se aportó, auto-avanza
FASE 3: load-estudio → detect → STOP ④ por módulos ambiguos (KEEP/STUDIO/NEUTRALIZE/TO_DROP)
FASE 4: compile → steps_generated en migration.yaml
FASE 5: iterate loop hasta -u all limpio

Cliente: <cliente>, v<version_origen>→v<version_destino>
Workdir: migration_kit_<cliente>/
PG iterate: localhost:5433 (Docker)
PG dev: localhost:5432 (local)
```

**Output:**
- `migration.yaml` compilado con decisiones por módulo
- `-u all` pasa sin errores en el contenedor Docker
- Dump final listo en local — **la subida al servidor es manual (BLOQUE 5)**

---

## BLOQUE 3 — Migración de código

> **Fin del hito autónomo "Entorno local listo"**: el port hasta arrancar `odoo-bin` limpio
> en local (sin ERROR/WARNING) + smoke RPC verde. Los MR (STOP ⑦) son POSTERIORES al hito.

### 3.1 Setup inicial

**Skill:** `developer-skills:dynapps-code-migration`

```
Usa developer-skills:dynapps-code-migration para el setup inicial.
Cliente: <cliente>, v<version_origen>→v<version_destino>.   Escenario: <scenario>
Repo nuevo: <repo>  |  Repo viejo: <repo_origen>
Buildout: $ODOO_SRC/buildouts/<cliente><version_destino>

Solo onpremise_gitlab: STEP 0.5 gitlab-prerequisites.md (histórico integrado + accesos).
odoosh_github: saltar gitlab-prerequisites.md; addons OCA vendorizados en parts/oca.

Pasos:
  - git remote add <version_origen> <repo_origen> + git fetch + git rebase
  - Si conflictos sin patrón → STOP ⑤
  - [MIG] Initial Commit (--no-verify — única excepción permitida en todo el flujo)
  - git push  (POSTERIOR al hito — requiere confirmación; fuera de la corrida autónoma)
  - Todos los módulos custom → installable:False + pre-commit exclude
  - indaws-migrate modules order → lista topológica
```

**Output:** lista ordenada de módulos custom para portar.

### 3.2 Loop por módulo (uno a la vez, en orden topológico)

**Port del código — `developer-skills:odoo-module-upgrader`:**
```
Usa developer-skills:odoo-module-upgrader para <modulo>.
v<version_origen> → v<version_destino>.
Feature branch: <ticket>_migrate_<modulo>_v<version_destino>
Buildout: $ODOO_SRC/buildouts/<cliente><version_destino>
Si quedan # TODO-PORT sin patrón → STOP ⑥.
```

**Iterate loop local (hasta el hito):**
```bash
indaws-migrate iterate --dump $DUMP_PATH/original.dump
# errores con receta → suggest --apply --commit (automático)
# errores sin receta → STOP ⑥
# módulo OK → smoke RPC → modules set <modulo> --status smoked
```

**MR + pipeline — POSTERIOR al hito, `developer-skills:git-manager` (provider auto-detectado):**
```
Usa developer-skills:git-manager para (fase de entrega, tras el hito local):
  1. pre-commit run -a → fix → commit → push  (cada push/PR/MR = confirmación explícita)
  2. Crear MR/PR por módulo:
     - onpremise_gitlab → glab, MR a develop, título "[MIG] <modulo> v<origen>→v<destino>"
     - odoosh_github    → gh, PR (sin pipeline GitLab; gate = pre-commit + tests locales)
     UN MR/PR POR MÓDULO. Nunca agrupar.
  3. onpremise_gitlab: monitorizar pipeline — esperar verde. Rojo → fix, nuevo commit, reintentar.
  4. Verde/gate OK → STOP ⑦: notificar al técnico; esperar merge manual antes del siguiente módulo.
```

**STOP ⑦ (MR/PR listo):**
```
"El MR/PR '[MIG] <modulo>...' está listo. Gate ✅
 URL: <url>
 El técnico debe revisar y mergear.
 Confirma cuando esté mergeado para continuar con el siguiente módulo."
```

### 3.3 Cierre

```bash
# end-migrate.py en <prefijo>_base  (uninstall stubs)
# end-reactivate_views.py en <prefijo>_all
# MR/PR de cierre — también con su gate (pipeline verde en GitLab / pre-commit+tests en GitHub)
```

---

## BLOQUE 4 — Validación

**Skill:** `common-skills:odoo-pilot`

```
Usa common-skills:odoo-pilot contra BD local migrada (localhost:5432).
Smoke RPC de todos los módulos migrados.
Generar checklist por módulo para STOP ⑧.
```

**STOP ⑧:** consultor valida funcionalmente.
- "No/Parcial" → volver BLOQUE 3 con scope del módulo concreto.
- Todos "Sí" → continuar BLOQUE 5.

---

## BLOQUE 5 — Go-live

**Ejecutado por:** el orquestador.

**Checklist automático antes de STOP ⑨:**
```bash
indaws-migrate modules list | grep -v -E "smoked|done|skipped"  # vacío
# onpremise_gitlab: glab mr list --state=opened --search="MIG"  # ninguno abierto
# odoosh_github:    gh pr list --state open --search "MIG"       # ninguno abierto
indaws-migrate ready                                             # gates verdes
```

**STOP ⑨:** mostrar resumen de gates + solicitar confirmación.

**Post-confirmación:**
```
onpremise_gitlab (MANUAL, fuera del flujo):
  - Subir dump migrado al servidor de producción (NAS/S3/SCP/dyn-cloud-manager)
  - El equipo de infra restaura el dump en el entorno de producción

odoosh_github:
  - Cutover nativo odoo.sh por commit (odoo-database-upgrader: playbooks/go_live/odoosh_cutover_commit.md)

AUTO (tras subida/cutover):
  - Smoke RPC en producción vía common-skills:odoo-pilot
  - Informe final de migración
```

---

## Tabla resumen de routing

| Condición | Skill / Acción | STOP |
|---|---|---|
| Máquina sin prerequisites | dynapps-dev-bootstrap | ① Vault login |
| Buildout no existe | dynapps-local-env-setup | ② Credenciales/repo |
| Docker iterate | orquestador directo | — |
| Upgrade Service BD | dynapps-db-upgrade-runner / odoo-database-upgrader | (test/prod + neutralización: gates) |
| Estudio + iterate loop | odoo-database-upgrader | ③ Estudio ④ Decisiones |
| Setup repo + rebase | dynapps-code-migration | ⑤ Conflicto rebase |
| Port módulo | odoo-module-upgrader | ⑥ TODOs sin patrón |
| **Hito "Entorno local listo"** | orquestador (fin corrida autónoma) | — |
| MR/PR + gate (entrega) | git-manager | ⑦ MR/PR listo → merge manual |
| Smoke RPC local | odoo-pilot | — |
| Validación funcional | orquestador | ⑧ Checklist consultor |
| Go-live + subida/cutover | orquestador + manual | ⑨ Confirmación final |
