# Fase 0 — Métodos de acceso a código y BD

> Este fichero detalla los **métodos concretos** de acceso. La decisión de **qué escenario** aplica
> (1 con fuente / 2 solo BD local / 3 producción) y qué se puede afirmar en cada uno está en
> `access-scenarios.md` — léelo primero. Aquí se describe sobre todo el **Caso 1** (código + BD) y el
> cómo levantar Postgres, válido también para el Caso 2 (sobre la copia desechable).

El Caso 1 (el más favorable) necesita los dos insumos. En casos 2/3 no hay código fuente y el motor
es `odoo-reverse-engineering`; aun así, casi siempre hay **BD** que leer con los métodos de abajo.

## 1. Código fuente (para el Pilar A — salud del código)

Cualquier método sirve, en orden de preferencia:

| Método | Qué pedir | Notas |
|--------|-----------|-------|
| Git | URL del repo + credenciales/deploy key, o un clon | Lo ideal: da historia y ramas |
| ZIP / tarball | Export del `addons` custom (no el core de Odoo) | Suficiente para auditar |
| Carpeta compartida | Acceso al `addons_path` del servidor | Verificar que están los módulos custom |
| Acceso al servidor | SSH al host | Permite también el dump de BD |

Mínimo aceptable: **los módulos custom** (no hace falta el core ni enterprise de Odoo,
que ya tenemos en `$ODOO_SRC*`). Sin código custom no hay sección 05.

## 2. Base de datos (para el Pilar B — Studio + acciones)

**No se restaura Odoo.** Solo se necesita poder **leer** la BD para extraer lo que vive fuera
de módulos (campos Studio, modelos custom, acciones de servidor y automatizadas, vistas
Studio).

**El método estándar es levantar SOLO Postgres en docker con el dump restaurado y consultar
las tablas `ir_*` por SQL. NO se levanta Odoo.**

| Método | Qué pedir | Notas |
|--------|-----------|-------|
| Dump → Postgres docker (estándar) | `.dump`/`.sql`/`.zip`/`.gz` de la BD | Restaurar con `docker-deployment` y levantar **solo el servicio `db`**. Sin app Odoo |
| Solo lectura SQL | Credenciales psql de una réplica | Si el cliente ya da acceso a la BD |
| API live (fallback) | URL + usuario + API key | Solo si ya hay un Odoo en marcha; vía `odoo-pilot` lectura |

### Levantar SOLO Postgres con el dump (delegado a `docker-deployment`)

No montamos el contenedor a mano: `docker-deployment` ya tiene Postgres dedicado y restore
multi-formato. Solo levantamos el servicio `db`, **nunca Odoo**.

```bash
# scripts de docker-deployment (common)
new.sh    <proyecto> <version_odoo> --no-ee   # genera el compose (db + odoo)
up.sh     <proyecto> db                        # levanta SOLO Postgres (pasa 'db' a compose up)
restore.sh <proyecto> <path-al-dump> --db-only # restaura y NO arranca Odoo (solo Postgres)
```

> **`--db-only` es obligatorio aquí:** sin él, `restore.sh` termina con `compose up -d odoo`
> y arranca Odoo — lo que viola el NEVER "solo Postgres" de esta skill. El flag deja únicamente
> el servicio `db` vivo para la extracción por SQL.

Tras el restore, leer del `.env` del proyecto el `DB_PORT`/`DB_USER`/`DB_PASSWORD` y el nombre
de la BD restaurada, y exportarlos para el extractor:

```bash
export PGHOST=localhost PGPORT=<DB_PORT> PGUSER=<DB_USER> PGPASSWORD=<DB_PASSWORD> PGDATABASE=<db>
```

> Da igual que el dump sea `.zip` (backup Odoo con `dump.sql`+`filestore/`), `.sql` plano,
> `.dump` (pg_dump -Fc) o `.gz`: `restore.sh` lo detecta. El filestore no se necesita para
> extraer Studio/acciones.

**Por qué solo Postgres:** las personalizaciones (campos Studio, modelos custom, acciones de
servidor/automatizadas, vistas Studio) viven en tablas `ir_*` y se leen por SQL. Arrancar el
servidor Odoo no aporta nada y exigiría los addons/dependencias del cliente que no tenemos.

## Checklist de arranque

- [ ] Tengo los módulos custom (vía git / zip / carpeta / servidor).
- [ ] Tengo la BD (dump restaurado en Postgres docker **o** acceso SQL/API solo-lectura).
- [ ] Sé la versión Odoo origen y la edición (CE/EE).
- [ ] Tengo (o no) versión objetivo para el gap de versión.
- [ ] Tengo las notas de la sesión de consultoría funcional (para la sección 06).

Si falta el **código** pero hay **BD**, no se bloquea: se pasa a **Caso 2/3** con
`odoo-reverse-engineering` (ver `access-scenarios.md`). Si no hay **ninguna BD**, o falta el
**consentimiento escrito** en Caso 3 → bloquear y avisar al comercial/consultor.
