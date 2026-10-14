# Checklist de Impacto Odoo (pasada obligatoria)

**Por qué existe.** Las 7 secciones de una tarea son genéricas. Las tareas de Odoo se rompen
casi siempre por lo mismo, y son cosas que un senior mira de memoria y un junior no. Esta pasada
convierte ese conocimiento implícito en un tachado explícito: **cada punto se marca `APLICA` (y
entra en la tarea) o `N-A` con motivo de una línea**. Un `N-A` sin motivo = no se ha mirado.

> Obligatorio para nivel de entrega **estándar** y **complejo** (`delivery-thresholds.json` →
> `requires_by_level`). En **complejo** cuenta como el `impact_analysis` que exige el nivel.
> En **trivial** se omite (fast-path).

Regla dura: **si un punto APLICA, la tarea debe decir qué hace el dev al respecto** — no basta con
marcarlo. Un "APLICA" sin instrucción es una mina para el dev.

---

## 1. Registros existentes / migración de datos

El olvido nº1 al añadir un campo `stored`/`computed` o cambiar un default.

- ¿El campo nuevo es `store=True`? → los registros existentes quedan a `NULL`/0 hasta recompute.
  ¿Hace falta init script (`_compute` masivo en post-init) o migración `pre/post`?
- ¿Se cambia el tipo/semántica de un campo con datos ya cargados? → plan de conversión.
- ¿Se renombra un campo/modelo? → nunca en caliente: script de migración + `oldname` handling.

**Si APLICA** → la tarea lleva instrucción explícita ("recalcular X registros existentes en
post-init") **y** el nivel de entrega sube a complejo (`has_data_migration: true`).

## 2. Multi-company

- ¿El modelo tiene `company_id`? ¿El campo/dominio nuevo lo respeta?
- ¿Dominios de búsqueda / defaults filtran por `company_id` (o `company_ids` en v18+)?
- ¿Reglas de registro (record rules) multi-company afectadas?

## 3. Multi-currency

- ¿Hay importes monetarios? → `currency_id` acompañante obligatorio, `Monetary` no `Float`.
- ¿Se compara/suma dinero de compañías o fechas distintas? → conversión con la fecha correcta.

## 4. Seguridad: ACL / record rules / groups

- ¿Modelo nuevo? → `ir.model.access.csv` obligatorio (sin él, nadie accede / traza en logs).
- ¿El campo muestra datos sensibles (costes, márgenes, RRHH)? → `groups=` en el campo/vista.
- ¿Se añade una record rule o se toca un dominio de una existente? → verificar que no abre acceso.
- ¿`sudo()` en el diseño? → justificar POR QUÉ y confirmar que no salta permisos que deben aplicar.

**Si APLICA** → `changes_security_or_acl: true` (sube a complejo) y la tarea nombra grupos/reglas.

## 5. Upgrade-safety (custom sobre core)

- ¿Se extiende/override un método o campo de un módulo core u Odoo estándar?
- ¿Sobrevive al próximo upgrade de versión? → preferir `super()` + hook, no copiar-pegar el método.
- ¿Se toca una vista estándar por `xpath`? → xpath frágil (por posición) vs robusto (por atributo).

## 6. Rendimiento a escala

- ¿`computed stored` sobre un modelo con muchos registros? → coste de recompute masivo.
- ¿El `_compute`/`_search` hace queries dentro de un bucle? → riesgo N+1.
- ¿Cron/batch? → tamaño de lote, idempotencia, qué pasa si peta a medias.
- ¿Se añade a un `depends` un campo que cambia mucho? → recomputes en cascada.

## 7. Concurrencia / integraciones externas

- ¿Llama a una API externa dentro de una transacción? → timeout bloquea el commit; aislar.
- ¿Webhook/import concurrente puede tocar el mismo registro? → locking / idempotencia.

---

## Verificación del módulo de referencia (para Junior/Mid)

Si la tarea apunta a un módulo de referencia como patrón (ALWAYS #3), **verificar antes de citarlo**:

- [ ] La ruta existe (`Glob`/`ls`) — no citar de memoria.
- [ ] Está a la **versión objetivo** de la tarea (un ref migrado mal contagia el bug al dev).
- [ ] El patrón citado es realmente el vigente ahí (no un método que a su vez está deprecated).
- [ ] Se cita el `path` concreto (y si es un método/campo, `file:line`), no "mira cómo lo hace X".

Un junior enviado a un ref inexistente o desactualizado pierde las mismas horas que sin referencia.

---

## Salida en la ficha

En la sección **"Impacto Odoo"** de la tarea: solo los puntos que **APLICAN**, cada uno con la
instrucción concreta para el dev. Los `N-A` no se listan (ya se han descartado en la pasada).
Si ninguno aplica en una tarea no-trivial → sospechar: probablemente no se ha mirado a fondo.
