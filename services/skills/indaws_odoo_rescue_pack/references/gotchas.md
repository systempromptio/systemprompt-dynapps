# GOTCHAS — Rescue Pack

0. **Esto NO es un informe de migración.** El objetivo es el **estado actual** del código y la BD
   (qué hay, salud, qué se hizo a mano). El coste de migrar de versión es un **extra opcional**
   (sección 07). No conviertas el informe en un plan de upgrade ni puntúes solo "dificultad de
   migración": la nota principal es la **salud** del código hoy.

0b. **La puntuación 0-10 es SALUD, 10 = MEJOR** (impecable). NO es riesgo/dificultad (no "10 =
   peor"). Más nota → más sano. Media alta = bien. "Sin candidatos a reescritura" = ningún módulo
   con salud **≤ 2** (no "≥ 8"). Colores: verde = alto/sano, rojo = bajo/problemático.

1. **Auditar solo el código fuente deja fuera media foto.** El partner anterior casi siempre
   dejó lógica en la BD vía Studio: campos `x_studio_*`/manuales, modelos `x_*`, acciones de
   servidor y automatizadas, vistas Studio. No está en `addons` ni en git. Sin extraerlo, la
   estimación de migración sale corta.

2. **Levanta SOLO Postgres, no Odoo.** Restaura el dump (zip/sql/dump/gz) con `docker-deployment`
   y arranca únicamente el servicio `db` (`up.sh <proj> db`). Extrae por SQL contra las tablas
   `ir_*`. Arrancar Odoo exigiría los addons del cliente y no aporta nada para esto.

3. **Las acciones llevan código Python en la BD.** `ir.actions.server.code` y las
   `base.automation` enlazadas contienen lógica real. Extráela (`extract_studio.sh` la vuelca a
   `.py`) y audítala como cualquier otro código (realimenta el Pilar A).

4. **Consolida en un único módulo `odoo_studio`.** Aglutinar el custom-fuera-de-módulos en un
   módulo lo saca de la opacidad de la BD: pasa a git, se vuelve auditable y migrable. Es el
   artefacto que se lleva a la migración.

13. **Filtra por XMLID o te ahogas en registros estándar.** `ir_act_server`/`ir_act_window`/
    `ir_ui_menu`/`ir_cron`/`base_automation` traen cientos de records de Odoo y de los módulos
    custom (ya en código). Solo interesa lo de runtime/Studio = sin XMLID de módulo en
    `ir_model_data` (el `where` del config ya lo aplica). Caso real: 127 server actions → 4.

14. **`name`/`field_description` son `jsonb` traducidos en v16+** (`{"es_ES":...}`), no texto.
    El script ya extrae `es_ES`/`en_US`; si consultas a mano, no asumas string plano.

5. **Distingue Studio de manual.** `ir.model.fields.state = manual` cubre tanto Studio
   (`x_studio_*`) como campos manuales `x_*` creados a mano. Ambos van a `odoo_studio`, pero
   decide por elemento (con `odoo-classification-guide`) qué mantener como data y qué
   reescribir como código nativo o sustituir por estándar/Indaws Core.

6. **Vistas Studio = `arch_fs = false`.** Su arch vive en BD, no en disco; por eso no aparece
   auditando el código. Extráelas explícitamente.

7. **Crons que apuntan a acciones de servidor.** `ir.cron.ir_actions_server_id` revela
   automatismos e integraciones. A menudo son la fuente de datos real de la instancia.

8. **Nunca escribas en la instancia del cliente.** Solo lectura: sin create/write/unlink.

9. **No presupuestes aquí.** El precio y el producto del Rescue Pack viven en `config.json`
   (`rescue_pack_*`); la propuesta la genera `quotation-builder`. Esta skill entrega diagnóstico
   + el módulo `odoo_studio`, no oferta.

10. **`odoo-code-auditor` ya tiene su `gotchas.md`** anti-falsos-positivos. No dupliques esa
    lógica: confía en la delegación.

15. **El alcance lo manda la BD, no el disco.** Auditar solo lo **instalado**
    (`ir_module_module.state`). En disco sobran módulos no instalados y árboles OCA enteros sin
    usar; auditarlos es trabajo tirado.

16. **Clasifica por `author`, no por carpeta.** Hay OCA vendorizado dentro del repo "custom"
    (caso real: `sale_layout_category_hide_detail` en `farmasoft/` pero autor OCA). Standard=
    `Odoo S.A.`, OCA=contiene `Community Association`/`(OCA)`. Ambiguos (`Camptocamp / Odoo`) los
    confirma el principal.

17. **OCA no se audita exhaustivo, solo continuidad** vs la última versión (hoy 19.0): ¿continúa
    la rama?, ¿absorbido por el core?, ¿descontinuado? Custom+Studio sí, a fondo.

18. **Instalado sin fuente = riesgo, pero pásale TODAS las rutas de addons.** Si solo das la ruta
    custom, todo OCA sale como "fuente perdida" (falso positivo). Incluye custom + OCA.

19. **"Sin fuente" ≠ caja negra.** Aunque no tengas los `.py`, `ir_model_data` atribuye cada
    modelo/campo/vista/menú/acción/grupo/regla a su módulo, e `ir_ui_view.arch_db` guarda el XML.
    `scripts/reconstruct_custom.sh` reconstruye por SQL la **capa declarativa completa** de cada
    custom (modelos, campos con ttype/relation/related/compute/store/depends + selection, vistas,
    seguridad, acciones con su código Python). Solo queda opaco el **Python imperativo** (cuerpos de
    compute/create/write/onchange, controladores) → `compute`/`store` en `ir_model_fields` marca
    dónde está. Es el motor del escenario dump-only sin addons, donde `odoo-reverse-engineering`
    (100% ORM, necesita Odoo vivo) no aplica.

20. **Dimensiona sin código con `db_intelligence.sh`.** Grafo de dependencias (orden/riesgo de
    migración), versión/estado por módulo, complejidad (nº modelos/campos/vistas), **fill-rate**
    (campo al 0% o modelo a 0 filas = candidato a NO migrar) y última actividad (vivo vs zombi).
    Convierte el modo solo-BD en una estimación de esfuerzo defendible sin leer una línea.

21. **`ir_config_parameter` contiene SECRETOS.** Certificados AEAT (`privateKey`/`publicCrt`),
    `client_secret` de Outlook/Google, tokens de integración. Extrae solo la **`key`** para mapear
    integraciones; **nunca vuelques los `value` en claro** al informe de cliente.

11. **El esquema `ir_*` cambia entre majors.** Algunas columnas/tablas no existen en versiones
    viejas (p. ej. `base_automation` separada de `ir.actions.server`, `arch_fs`/`arch_db`). El
    script marca `N/A` la tabla ausente y sigue; ajusta `config.json` si hace falta.

12. **Fallback API solo si ya hay Odoo live.** Si en vez del dump usas `odoo-pilot`, ojo: <19
    es JSON-RPC y 19+ JSON2. La vía estándar es el dump en Postgres por SQL.

22. **Dos documentos, dos públicos (estándar del equipo).** El **ejecutivo** (Documento A, 6
    secciones) es para gerencia: qué está mal y cuánto cuesta, SIN `file:line`. El **anexo técnico**
    (Documento B) es para el técnico del cliente: detalle módulo por módulo, flujos, bugs de runtime
    CON `file:line`. El ejecutivo **referencia** al anexo (coletilla en la tabla de salud 0-10). Se
    genera A+B por defecto (`{detalle}`=completo); `ejecutivo` = solo A. Adaptar la entrega al público
    es parte del trabajo, no un extra.

23. **Encuadre: ¿en producción o pre-arranque?** Si el cliente aún NO opera (implantación en curso,
    migración de datos pendiente), NO describas "estado de la BD" ni conteos como si fuera producción
    ni digas "no se ha probado porque está vacío". Encuadra: "los defectos viven en caminos aún no
    ejercitados; se manifestarán al arrancar con datos reales → resolver antes del go-live". La
    sección 2 pasa a "Composición del desarrollo a medida".

24. **Mapeo a Indaws Core con precios.** Cada custom problemático puede tener un paquete **Indaws
    Core** que lo sustituye (catálogo en `references/shared/paquetes_indaws_core.md`, 42 paquetes con precio). Los importadores/migradores custom → **Framework
    de Migración e Integración de Datos**. El **plan** de migración de datos NO se hace aquí: es
    observación para la etapa de análisis.

25. **Tooling PDF — tablas anchas y rutas.** (a) NUNCA una tabla de **4+ columnas con `<code>` largos**
    (rutas/nombres de módulo): desborda el A4 y corta la última columna → usa **bloques verticales**.
    (b) `word-break:break-word;overflow-wrap:anywhere` en la regla `code`. (c) Los scripts de
    `dynapps-brand` (`gen.js`/`stamp.py`) pueden traer **rutas absolutas hardcodeadas** (p. ej.
    rutas absolutas del entorno de build): ajústalas a la instalación real antes de renderizar. (d) `verify.py` NO
    detecta desbordamiento horizontal → **renderiza a PNG y revísalo a ojo** antes de entregar.

26. **Racionalización: NO recomendar "reducir a Studio".** Al analizar la necesidad funcional de un
    módulo custom, el veredicto para micro-módulos de solo-campos es **"consolidar/simplificar en
    código"** (fusionar en un módulo versionado o junto al que los consume), **nunca Studio**: Studio
    los volvería opacos y no versionables — justo lo contrario a lo que persigue la Fase 3b con
    `odoo_studio` — y desacopla campos que **alimentan lógica** de otros módulos. Studio solo, como
    mucho, para campos puramente informativos que el usuario final quiera tocar. Delegar el juicio
    Studio-vs-código a `odoo-classification-guide`.

27. **Patrón a cazar en Fase 2b: vencimientos/importes reescritos fuera de su mecanismo estándar.**
    Un módulo que sobreescribe `date_maturity` de apuntes **ya posteados** (en `_post`/`write`) en vez
    de dejar que lo derive el **plazo de pago** (`account.payment.term`) rompe la coherencia
    factura↔plazo, **entra en conflicto** con cualquier otro módulo (Core/OCA) que recalcule desde el
    término, y es peligroso si la factura está conciliada/remesada o con contabilidad inalterable
    (VeriFactu/SII). Regla general: la lógica contable va en su mecanismo nativo (plazo de pago,
    posición fiscal…), no parcheando el asiento. Igual con precios: reemplazar `_compute_price_unit`
    sin `super()` anula pricelists.

## El criterio "sin XMLID" corta por los dos lados (medido 2026-09-02)

`extract_studio.sh` identifica lo hecho a mano por **ausencia de XMLID en `ir_model_data`**. En un
rescate real ese criterio falló en las dos direcciones a la vez:

**Cuenta de más — registros que crea el propio Odoo en runtime.** Salieron 7 `ir.sequence` como
"personalización". Ninguna se hizo a mano: cinco las genera Odoo al dar de alta un `stock.warehouse`
(una por tipo de operación), otra es la de desechos y la séptima la crea `l10n_es_aeat`. Migrarlas
habría duplicado secuencias. **Antes de dar por personalización un registro sin XMLID, mira qué es.**

**Cuenta de menos — registros del núcleo editados en sitio.** Estos **sí tienen XMLID**, así que el
filtro los descarta; y como no viven en ningún módulo, ningún análisis de código los ve. En ese mismo
rescate había **21 vistas del core con el `arch_db` reescrito a mano**, 19 de ellas la vista base (no
una herencia): `sale.view_order_form`, `account.view_move_form`, `crm.crm_lead_view_form`,
`event.view_event_form`, `project.edit_project`, `subscription_oca.sale_subscription_form`… **Odoo
restaura sus propias vistas en cada `-u`**, así que se pierden enteras en la primera actualización —
y una migración actualiza todos los módulos. Es un coste de migración que no aparece en ninguna
estimación basada en el código.

Detección (añádela siempre a la Fase 3):
```sql
SELECT d.module||'.'||d.name AS xmlid, v.type,
       CASE WHEN v.inherit_id IS NULL THEN 'VISTA BASE' ELSE 'herencia' END AS clase,
       v.write_date::date
FROM ir_ui_view v
JOIN ir_model_data d ON d.model='ir.ui.view' AND d.res_id=v.id
WHERE v.write_uid IS NOT NULL AND v.write_uid <> 1
  AND v.write_date > d.create_date + interval '1 day'
ORDER BY clase, 1;
```

## El restore NEUTRALIZA los crons: la copia miente sobre producción

`docker-deployment restore.sh` desactiva crons, servidores de correo e integraciones — que es lo
correcto para no disparar nada. Efecto colateral: **`ir_cron.active` de la copia no dice qué corría
en el cliente**. En el rescate medido, la copia daba los tres crons como inactivos y el dump original
tenía uno **activo cada 6 horas corrompiendo el NIF de los contactos**.

Para conocer el estado real, restaura sólo esas tablas a una BD aparte:
```bash
pg_restore -U odoo -d <bd>_orig -t ir_cron -t base_automation -t ir_mail_server --no-owner <dump>
```
`lastcall` y `nextcall` sobreviven a la neutralización y ya delatan bastante: un `nextcall` en el
pasado sin `lastcall` posterior = el cron se desactivó en producción, no que fallara.

## Los OCA de disco NO son los que el cliente tiene instalados

La auditoría lee `/opt/OCA/v<N>`, que es la rama al día. **La BD dice la versión real**
(`ir_module_module.latest_version`). En el rescate medido había hasta 3 versiones menores de
desfase, y eso produjo un **falso positivo**: se marcó como bug un dominio que filtraba por
`state='sent'` porque la rama actual usa `0_sent`/`1_draft`/`2_signed` — pero la versión instalada
usaba los valores planos y el código era correcto. Contrasta siempre la versión instalada antes de
declarar roto algo que dependa de un módulo de comunidad.

## Un modelo con 0 filas no prueba nada si es transitorio

Antes de declarar código muerto por fill-rate, comprueba `ir_model.transient`: los `TransientModel`
(asistentes) los vacía el autovacuum, así que estar a cero es su estado normal. En el rescate medido
esto tumbó la conclusión de que un módulo entero estaba sin usar: sus tres modelos eran asistentes.
