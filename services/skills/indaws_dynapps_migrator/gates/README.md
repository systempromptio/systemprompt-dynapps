# Capa de gates de migración (flujo Bélgica)

Gates que validan una migración Odoo del flujo Bélgica. **Vive fuera del
`indaws_migration_kit` a propósito**: el kit es el motor técnico de BD y lo comparten
otras migraciones (España) que no siguen estos lineamientos. Aquí van las reglas
propias de este flujo; el kit se **importa**, nunca se modifica.

## Dos plataformas, una sola biblioteca

| | dyncloud BE | Odoo.sh |
|---|---|---|
| CI propio, pre-commit, estructura buildout | sí | no |
| MR con etiqueta y revisor | sí | PR, sin etiquetas |
| Control del envío al Upgrade Service | sí | lo lanza la plataforma |

Los gates **declaran qué capacidades necesitan** y el runner los salta con motivo
cuando el perfil no las tiene. Regla dura: **saltado no es verde**. Se reporta
`not_applicable` con la razón, para que una migración en Odoo.sh no acabe menos
validada sin que nadie lo note.

## Qué NO se asume

- Que exista un ticket. Odoo.sh puede no tenerlos: en el baseline y en las notas el
  motivo es obligatorio y el ticket opcional.
- Que exista pipeline. El `checklog` se ejecuta en local sobre el log de tests.
- Que exista estructura `xx_all` / `modules_to_remove` / `migrations`. Se declara en
  el perfil y los gates que dependen de ella se saltan con motivo.

## Cómo se ejecuta

```python
from gates.profile import MigrationProfile
from gates.registry import ALL_GATES
from gates.runner import GateContext, GateRunner

profile = MigrationProfile.load(workdir)
ctx = GateContext(workdir=workdir, profile=profile,
                  source_db=ref, target_db=ours, repo_roots=[repo / "parts/custom"],
                  installed_versions={"test": {...}}, log_path=run_log,
                  delivery=delivery, dump_taken_at="2026-08-16")
report = GateRunner(ctx, ALL_GATES).run()
report.write(workdir, profile)      # gates/report.md + report.json + un json por gate
```

El informe distingue cuatro cosas y esa es toda su gracia: lo que pasó, lo que falló,
**lo que esta plataforma no puede validar** (con su motivo y su compensación) y **lo que
sí aplica aquí pero no se ejecutó** por falta de insumo.

Los dos últimos no son lo mismo, y confundirlos es lo que produce el peor fallo posible:
una fase en verde sin haber comprobado nada. `not_applicable` es un hueco declarado — en
Odoo.sh no habrá pipeline nunca — y no bloquea. `not_run` es una ejecución rota: faltó
la conexión, la raíz del repo o el log. **Bloquea igual que un fallo.**

## Los gates

| Gate | Necesita | Qué detecta |
|---|---|---|
| `neutralized` | BD migrada | Correo, crons y conectores vivos — **incluidos los de flag no estándar** (`is_active`), por donde se colaron los que enviaban de verdad |
| `views-inactive` | BD migrada + Context del kit | Vistas custom que el upgrade dejó inactivas (envuelve el validador del kit) |
| `required-emptied` | BD migrada + Context del kit | Configuración required que salió vacía (envuelve el validador del kit) |
| `value-parity` | BD origen + migrada | Campos propios que llegan vacíos aunque existan y la tabla esté llena |
| `module-versions` | repo | `migrations/` que no se ejecutará: por encima del manifiesto, o por debajo de lo que ya trae instalada la BD que sube. En modo `initial` exige además que **nadie haya bumpeado**: todo módulo portado en su versión base |
| `checklog` | log de ejecución | ERROR/WARNING no ignorados; en Odoo.sh, el único control que hay |
| `data-files-freshness` | repo | Ficheros de datos de una migración extraídos de un snapshot anterior al dump que va a producción |
| `fresh-replay` | evidencia | Que el ensayo completo se hizo sobre un dump al menos igual de reciente, y salió verde |
| `delivery-hygiene` | perfil + entrega | Rama, co-autor y nota siempre; etiquetas, revisor y ticket solo donde existen |

`delivery-hygiene` corre **solo en la última fase**, después de la validación. El port de
módulos no publica nada: su gate es que el entorno local arranque con todo instalado y el
log limpio (`checklog` + `module-versions`). Si un gate de entrega se ejecutara durante el
port, estaría exigiendo una subida que aún no se ha decidido.

`module-versions` tiene dos modos y no son intercambiables. En `initial` -- el port a la
versión nueva -- **no se bumpea nada**: cada módulo se queda en su versión base, porque el
número al que se llegó subiéndolo a mano en local no existe en el go-live, y una reparación
de datos colgada de ese número no se ejecutará allí. En `maintenance` -- sistema ya vivo --
la regla se invierte: hay que superar lo instalado o el despliegue no actualiza. Por eso, en
`initial`, las versiones instaladas que se le pasan deben venir de la **BD recién migrada
del ensayo**, nunca de la BD local sobre la que se estuvo corrigiendo.

Los adaptadores (`adapters.py`) son el único sitio que sabe de `glab` y `gh`.

## Lo que se reutiliza del kit

`views-inactive` y `required-emptied` **no reimplementan nada**: ejecutan el
`PostRestoreValidator` del kit en modo detección (`apply=False`) y traducen su salida al
contrato de gates. Así hay un solo sitio donde vive esa lógica y un solo informe donde
se lee todo.

`neutralized` sí es propio: el kit tiene los pasos que **neutralizan**, y esto es la
comprobación de solo lectura de que quedó hecho.
