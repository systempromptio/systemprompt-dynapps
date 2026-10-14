# Verificación de Tests (Step 4) — innegociable

Regla del arquitecto: **sin tests no hay entrega**. Nosotros exigíamos tests unitarios + flujos
end-to-end que cubran los criterios de aceptación del MD. Aquí se comprueba que existen, cubren y
**pasan de verdad**.

---

## 1. Existen y son los correctos

- **Unitarios**: uno (o más) por **cada criterio de aceptación** listado en el MD. Un criterio sin
  test correspondiente = **correcciones**, aunque el código parezca funcionar.
- **End-to-end**: al menos un flujo que ejercita el caso de negocio completo (crear → operar →
  verificar el resultado que ve el usuario). Es el olvido más común: traen unit y no e2e.

## 2. Cobertura de criterios (la matriz)

Construir la correspondencia criterio → test:

| Criterio de aceptación (MD) | Test que lo cubre | ¿Pasa? |
|-----------------------------|-------------------|--------|
| Cuando coste 50 / precio 100 → 50% | `test_margin_basic` | ✅/❌ |
| Cuando precio 0 → 0% (no div/0) | `test_margin_zero` | ✅/❌ |
| Flujo: pedido → línea → margen | `test_tour_margin` (e2e) | ✅/❌ |

Cualquier fila sin test o en ❌ → no hay merge.

## 3. Verde REAL en el Docker limpio, no declarado

- **No fiarse de "los tests pasan" ni del CI**. Se ejecutan en el **Docker del Step 1**
  (`docker-verification.md`), con la versión objetivo y, si aplica, el dump del cliente:
  ```bash
  # contra la db del Docker de review (añadir el filtro de logs habitual)
  # El binario es `odoo`, no `odoo-bin`; puertos alternativos obligatorios.
  docker exec <proj>_odoo odoo -d <db_review> -u <modulo> \
    --test-enable --test-tags /<modulo> --stop-after-init \
    --http-port=8199 --gevent-port=8299 --max-cron-threads=0 2>&1 | tail -40
  ```
- Capturar el output y adjuntarlo a la decisión (verde/rojo real, no promesa).
- Si el PR no trae e2e claro → probar el flujo en el Docker a mano o con `odoo-flow-tester`.
- Tests que existen pero están **skippeados** o vacíos (`pass`, sin asserts) = como si no estuvieran.

## 3b. La suite ESTÁNDAR: cuándo vale la pena y cómo se compara

Medido sobre 13 PR de gobierno en un día: **7 corridas, 4 con delta vacío**. La suite estándar no
se corre "por rigor", se corre cuando el módulo puede alcanzarla.

**Cuándo NO** — el módulo solo quita permisos (ACL, record rules) y nada más: sin modelos, sin
datos, sin vistas. Odoo ejecuta sus suites **como administrador**, y estos módulos meten
`base.group_system` en `implied_by_ids` de su grupo para no dejarlo fuera, así que el candado no
puede tocarlas. Comprobación barata de que la teoría se cumple: **contar el `AccessError` propio
en el log estándar** — si sale 0, no ha saltado ni una vez.

**Cuándo SÍ**, sin excusa:

| el módulo… | por qué alcanza al estándar | medido |
| --- | --- | --- |
| escribe datos (`ir.default`, `post_init_hook`) | cambia lo que se crea, no quién puede | 27 fallos nuevos, 23 del default |
| pone un campo `required` | el estándar crea registros sin él | 613 tests ejecutados → **229** |
| toca una vista | un tour corre como un usuario concreto | 2 errores, 10 tests sin arrancar |
| restringe a un **usuario normal** en un modelo que el estándar ejercita como tal | ahí el admin no está exento | To-do y `project.task.type` |

**El control tiene que llevar el MISMO juego de módulos.** Comparar contra una base pelada
atribuye a este PR los fallos de todo lo que arrastra. Si el módulo tira de otros 26, el control
los lleva también: se comprueba con `SELECT count(*) FROM ir_module_module WHERE state='installed'`
a los dos lados — deben diferir en **exactamente una fila**.

**Dirigir por clase antes que correr la suite entera.** En un PR que ensanchaba la lectura de
`hr.employee`, 166 tests tardaban 90 minutos; **19 tests de las 5 clases que podían detectarlo**
dieron los mismos 2 fallos en 2 minutos. Se eligen leyendo qué asierta el estándar sobre lo que el
módulo cambia (`grep -l` por el modelo en `addons/<app>/tests/`), no por intuición.

**Un baseline sucio enmascara deltas.** Un control con 60 tests en rojo no distingue un delta de 5
del ruido: hoy ocultó una clase entera (`TestKnowledgeArticleConstraints`, 6 errores) que en un
control limpio salta a la vista. Si tu control no está a 0, arréglalo antes de sacar conclusiones.

**Refresca el head justo antes de medir.** Los números se publican con el sha delante. Hoy revisé
un head de tres horas antes y publiqué un recuento equivocado: la autora había subido un tercer
módulo entre mi `fetch` y mi lectura, y el defecto era mayor de lo que dije. `git fetch --force` la
rama y `git log -1` en el informe.

## 4. Calidad de los tests (no solo cantidad)

- El test **falla si el código falla**: si al romper el código a propósito el test sigue verde, no
  prueba nada.
- Assert sobre el **resultado esperado con valores concretos**, no "no explota".
- e2e que verifica el **estado final de negocio**, no solo que la página carga.

### 4.1 Falsación del test (paso estándar en bug-fix)

Un test de regresión que **pasa con y sin el fix** es humo: no cierra el bug, solo lo acompaña. Para
demostrar que **pesca el bug de verdad**, revertir SOLO el fix (dejando el test nuevo) y confirmar que
**FALLA** con la aserción esperada. Con una rama del PR ya checkouteada (ver `docker-verification.md`):

```bash
# 1) fix aplicado → el nuevo test PASA
docker exec <proj>_odoo odoo -d <db> -u <modulo> --test-enable \
  --test-tags '/<modulo>:<Clase>.<test>' --stop-after-init \
  --http-port=8199 --gevent-port=8299 2>&1 | tail -20

# 2) revertir SOLO el/los fichero(s) del fix, DEJANDO el test nuevo
git checkout <base> -- <ruta/al/fichero_del_fix.py>   # NO tocar el fichero de tests

# 3) re-ejecutar el mismo test → debe FALLAR con la aserción esperada
docker exec ... odoo ...   # mismo comando; buscar 'FAIL: ...' + AssertionError del propio test

# 4) restaurar el fix (dejar el árbol como estaba)
git checkout <rama_pr> -- <ruta/al/fichero_del_fix.py>
```

Si en el paso 3 el test sigue verde → el test no cubre el bug: **correcciones** (o el fix no era el
cambio real). Adjuntar a la decisión el `FAIL` real, no solo el verde final.

---

## Salida

Veredicto de tests: **cobertura completa + verde real** (habilita merge) o la lista de criterios sin
test / tests en rojo / e2e ausente (bloqueantes para Step 5).
