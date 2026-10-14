# Stop Points — Dynapps Migrator

Los 9 STOP points son los únicos momentos en que la skill pausa y espera intervención humana.
Fuera de estos puntos, todo es automático.

> **Escenarios**: on-premise/GitLab (Bélgica dyncloud + España) y odoo.sh/GitHub. STOP ②
> resuelve el hosting del repo si no se dedujo antes; el `scenario` gobierna el resto.

## Comportamiento en modo autónomo

La corrida autónoma (BLOQUES 1–3 hasta el hito "Entorno local listo", ver `autonomous-mode.md`)
solo se detiene en los STOPs **irreducibles**; el resto auto-avanza:

| STOP | Autónomo |
|---|---|
| ① vault-login | **SE MANTIENE** (OIDC navegador) |
| ② buildout/repo | AUTO-AVANZA (escenario ya resuelto; IDE se salta) |
| ③ estudio previo | AUTO-AVANZA si se aportó/auto-detecta |
| ④ módulo ambiguo | **SE MANTIENE** (decisión de negocio) |
| ⑤ rebase sin patrón | **SE MANTIENE** (decisión técnica) |
| ⑥ TODO sin patrón | **SE MANTIENE** (decisión técnica) |
| ⑦ MR/PR merge | FUERA (posterior al hito) |
| ⑧ validación funcional | FUERA (posterior al hito) |
| ⑨ go-live | FUERA (posterior al hito) |

Además, dos gates NO numerados que **se mantienen** en autónomo: la decisión **test/producción**
del Upgrade Service (odoo.sh siempre test → sin STOP) y el **gate de neutralización en rojo**.

---

## STOP ① — Login Vault/OIDC

**Bloque:** 1 — Preparación  
**Skill que lo genera:** `developer-skills:dynapps-dev-bootstrap`  
**Cuándo:** La máquina tiene prerequisites pero Vault no tiene sesión activa  
**Detección:**
```bash
vault token lookup 2>/dev/null || echo "VAULT_NO_SESSION"
```

**Mensaje al usuario:**
```
STOP ① — Login requerido en Vault

La máquina está preparada pero no hay sesión activa en Vault/OIDC.

Abre un terminal y ejecuta:
  ! vault-login

Cuando el login sea exitoso, responde "listo" para continuar.
```

**Cómo continuar:** Usuario responde → verificar `vault token lookup` → si OK, continuar Bloque 1.

---

## STOP ② — Buildout no existe

**Bloque:** 1 — Preparación  
**Skill que lo genera:** `developer-skills:dynapps-local-env-setup`  
**Cuándo:** El buildout del cliente no existe en `$ODOO_SRC/buildouts/<cliente><version>/`

**Mensaje al usuario:**
```
STOP ② — Buildout no encontrado

No existe buildout para <cliente> v<version_destino> en $ODOO_SRC/buildouts/<cliente><version_destino>/.

Voy a crearlo con setup-local-environment. Necesito confirmar:
  - Repo GitLab: <repo detectado o inferido>
  - ¿El repo está en GitLab Dynapps? [Sí / No — está en GitHub DynAppsNV]

Responde para continuar.
```

**Cómo continuar:** Recibir confirmación → ejecutar `setup-local-environment` → continuar Bloque 1.

---

## STOP ③ — Estudio previo del consultor

**Bloque:** 2 — BD side  
**Skill que lo genera:** `developer-skills:odoo-database-upgrader`  
**Cuándo:** Antes de `load-estudio`, después de `indaws-migrate init`

**Mensaje al usuario:**
```
STOP ③ — Estudio previo requerido

El workdir está creado. Antes de continuar necesito el estudio previo
del consultor con las decisiones por módulo.

Formatos aceptados: .md / .xlsx / .pdf
Ubicación esperada: ~/migrations/migration_kit_<cliente>/estudio_previo.*

¿Tienes el estudio? Puedes:
  a) Arrastrar el fichero al chat
  b) Indicar la ruta donde está
  c) Si no hay estudio, continuar con detección automática (recomendado solo en proyectos pequeños)
```

**Cómo continuar:**
- Con estudio → `indaws-migrate load-estudio <path>` → continuar Bloque 2
- Sin estudio → `indaws-migrate detect --auto` → ir directo a STOP ④

---

## STOP ④ — Decisión por módulos ambiguos

**Bloque:** 2 — BD side  
**Skill que lo genera:** `developer-skills:odoo-database-upgrader`  
**Cuándo:** Después de `detect`, por cada módulo sin decisión clara

**Decisión = portar o no** por módulo: KEEP (portar el código) / STUDIO / NEUTRALIZE / TO_DROP
(no portar → stub para desinstalar). Es la decisión que el pre-flight ya intenta recabar del
estudio previo (`requirements.md`); aquí caen solo los módulos que quedaron sin decisión clara.

**Default de comportamiento = 1:1.** Cuando la duda es por una feature que cambió en la v-destino,
el default es **conservar el comportamiento del cliente** (KEEP + portar la funcionalidad tal cual
en v-origen), no adoptar el nuevo estándar de Odoo. Desviarse = decisión de negocio explícita.

**Estrategia de agrupación:** Presentar por lotes del mismo tipo (todos los OCA juntos, todos los custom juntos). Reducir al mínimo el número de prompts.

**Mensaje al usuario (por lote):**
```
STOP ④ — Decisión requerida para módulos ambiguos

Los siguientes módulos no tienen decisión clara en el estudio previo.
Para cada uno, indica: KEEP / STUDIO / NEUTRALIZE / TO_DROP

Módulos OCA (posiblemente disponibles en v<version_destino>):
  - sale_order_revision (OCA/sale-workflow) → ¿KEEP / TO_DROP?
  - stock_picking_batch_extended (OCA/stock-logistics) → ¿KEEP / TO_DROP?

Módulos custom (requieren port):
  - tecniman_sale_custom → ¿KEEP / NEUTRALIZE / TO_DROP?
  - tecniman_report_custom → ¿KEEP / NEUTRALIZE / TO_DROP?

Módulos de terceros (sin mantenimiento conocido):
  - old_module_xyz → recomendación: NEUTRALIZE. ¿Confirmas?

Puedes responder: "KEEP los OCA, NEUTRALIZE tecniman_report_custom, TO_DROP old_module_xyz"
```

**Cómo continuar:** Parsear respuesta → registrar decisiones en migration.yaml → si quedan módulos sin decisión, repetir STOP ④ → cuando todos tengan decisión, ejecutar `compile`.

---

## STOP ⑤ — Conflictos de rebase sin patrón

**Bloque:** 3 — Code migrate  
**Skill que lo genera:** `developer-skills:dynapps-code-migration`  
**Cuándo:** Durante `git rebase` aparecen conflictos que el ruleset automático no puede resolver

**Mensaje al usuario:**
```
STOP ⑤ — Conflicto de rebase sin patrón automático

El rebase de <fichero> tiene un conflicto que no reconozco:

--- CONFLICT: <path/al/fichero.py> ---
<<<<<<< HEAD (v<version_destino> base)
[código nuevo]
=======
[código del cliente v<version_origen>]
>>>>>>> [commit-hash] ([descripción])

¿Cómo quieres resolverlo?
  a) Mantener la versión v<version_destino> base (ours)
  b) Mantener la versión del cliente (theirs)
  c) Mostrar diff completo del fichero para decidir
  d) Marcar como TODO y continuar (se creará issue en el tracker)
```

**Cómo continuar:** Según respuesta → aplicar resolución → `git rebase --continue` → si hay más conflictos, repetir.

---

## STOP ⑥ — TODOs sin patrón tras port de módulo

**Bloque:** 3 — Code migrate (loop por módulo)  
**Skill que lo genera:** `developer-skills:odoo-module-upgrader`  
**Cuándo:** Después del port automático de un módulo quedan TODOs no resueltos

**Mensaje al usuario:**
```
STOP ⑥ — Port de <módulo> completado con TODOs pendientes

El port automático de <tecniman_sale_custom> dejó N TODOs:

  1. [TODO] models/sale_order.py:45 — action_confirm: el método cambió
     de firma en v<version_destino>, revisar compatibilidad
  2. [TODO] views/sale_order_view.xml:23 — campo obsoleto 'x_field_old'
     eliminado en v<version_destino>, buscar equivalente

¿Quieres:
  a) Resolver ahora (te guío uno a uno)
  b) Marcar como revisados y continuar (asumes la responsabilidad)
  c) Escalar al equipo dev con nota de diagnóstico
```

**Cómo continuar:** Según respuesta → resolver inline / continuar / crear issue → actualizar estado del módulo → continuar el loop.

---

## STOP ⑦ — MR listo (pipeline verde, merge manual)

**Bloque:** 3 — Code migrate (tras MR/PR) — POSTERIOR al hito autónomo  
**Skill que lo genera:** `developer-skills:git-manager`  
**Cuándo:** El gate del MR/PR está verde (pipeline GitLab en `onpremise_gitlab`; pre-commit +
tests locales en `odoosh_github`) y el módulo está listo para merge

**Mensaje al usuario:**
```
STOP ⑦ — MR/PR listo para revisión y merge

Listo:
  Título: [MIG] <módulo> v<version_origen>→v<version_destino>
  URL:    <url>
  Gate:   ✅ (pipeline verde / pre-commit+tests locales)

Por favor revisa y mergea el MR (GitLab) / PR (GitHub).
Cuando hayas hecho el merge, responde "mergeado" para continuar con el siguiente módulo.
```

**Cómo continuar:** Usuario confirma merge → `indaws-migrate modules set <module> --status verified` → `git checkout develop && git pull` → siguiente módulo.

Si la pipeline falla antes de llegar aquí → diagnosticar y corregir, nunca notificar con pipeline roja.

---

## STOP ⑧ — Validación funcional del consultor

**Bloque:** 4 — Validación  
**Skill que lo genera:** `common-skills:odoo-pilot` (smoke RPC completado)  
**Cuándo:** Smoke RPC automático OK, pero se requiere validación funcional humana

**Antes de presentarlo:** si la Capa 2b (navegador) corrió, sus **HALLAZGOS** y sus flujos
**NO CONCLUYENTES** se incorporan al checklist — el consultor no debería redescubrir a mano
lo que la capa ya señaló, ni dar por bueno lo que quedó sin concluir.

**Mensaje al usuario:**
```
STOP ⑧ — Validación funcional requerida

El smoke RPC automático pasó. Necesito que el consultor valide
funcionalmente la BD migrada en el entorno local.

Checklist por módulo (responde Sí / No / Parcial para cada uno):

Módulos core:
  - [ ] sale: Crear presupuesto, confirmar, facturar
  - [ ] purchase: Crear PO, recibir, facturar proveedor
  - [ ] stock: Mover stock, inventario, valoración
  - [ ] account: Asientos, cierres, conciliación

Módulos custom migrados:
  - [ ] <modulo_custom_1>: <flujo específico del cliente>
  - [ ] <modulo_custom_2>: <flujo específico del cliente>

URL local: http://localhost:8069
Usuario: admin / Contraseña: admin

Ya detectado por la validación en navegador (Capa 2b) — confirmar:
  - [ ] <hallazgo_1>: <pantalla> — <qué se observó>
  - [ ] <flujo no concluyente_1>: <pantalla> — sin veredicto automático

Los módulos marcados como "No" o "Parcial" volverán al Bloque 3 para corrección.
```

> La contraseña `admin` solo es válida si se fijó a mano: en un clon de producción el admin
> **no** la tiene por defecto.

**Cómo continuar:** Parsear respuestas → módulos "No/Parcial" → Bloque 3 con scope reducido → cuando todos "Sí" → Bloque 5.

---

## STOP ⑨ — Confirmación go-live

**Bloque:** 5 — Go-live  
**Skill que lo genera:** dynapps-migrator (directamente)  
**Cuándo:** Todos los gates automáticos están en verde

**Mensaje al usuario:**
```
STOP ⑨ — Confirmación de go-live

Todos los gates están verificados:

  [✓] Smoke RPC en local OK
  [✓] Todos los módulos en estado "smoked"
  [✓] Gate verde en todos los MR/PR (pipeline GitLab / pre-commit+tests locales)
  [✓] MR/PR mergeados en develop

Resumen de la migración:
  - Cliente: <cliente>
  - Versión: v<version_origen> → v<version_destino>
  - Módulos migrados: N
  - Duración total: X días
  - Issues resueltos: N
  - Issues escalados a dev: N

El dump migrado está en: <ruta_local>
La subida al servidor de producción es manual.

¿Confirmamos go-live? [Sí / No / Posponer]
  "Sí" → informe final generado
  "No" → se aborta (el estado queda guardado)
  "Posponer" → indica fecha/hora deseada
```

**Cómo continuar:** "Sí" → generar informe final de migración. "Posponer" → guardar estado.
