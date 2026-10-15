# Carril Upgrade Lab (migraciones grupo Dynapps)

Carril ligero para las migraciones que Indaws realiza para el grupo Dynapps.
Fuente de verdad: **artículo Knowledge 2602, sección "Upgrade Lab"**. Esta referencia
solo describe cómo lo materializa el skill.

Se activa con `profile == upgrade-lab` (Paso 0). Diferencias clave frente al interno:
servidor lo pone el cliente (sin `[INFRA]`), no hay kickoff (informe de módulos),
migración **siempre 1:1** (sin Studio), el cliente testea y despliega (sin testing
funcional/rehearsal/Go-Live nuestros), cierre = corregir feedback + re-migración final.

---

## Estructura: 8 tareas (4 MIG + 3 GATE + 1 gestión)

| Fase | Tarea | Gate al cierre | Horas |
|---|---|---|---|
| UL-1 | `[MIG] Análisis e informe de módulos` | Gate A | `--analysis-hours` |
| UL-2 | `[MIG] Migración de código` (1:1) | — | suma del inventario |
| UL-3 | `[MIG] Migración de base de datos` | Gate B (entrega) | `--db-hours` |
| UL-4 | `[MIG] Corrección de feedback y re-migración final` | Gate C (cierre/producción) | `--fix-hours` |
| — | `Gestión del proyecto` (imputación) | — | 0 |

- Las 4 tareas `[MIG]` llevan el tag **Migration**; la de gestión no.
- **NUNCA** se crea "Reuniones" (no hay kickoff ni dailies).
- Gates A/B/C son **internos**: aprueba Tech Lead (PM en el cierre). Sin COO/CTO/cliente.

### Checklists por tarea (los crea el motor)

- **UL-1**: analizar + redactar informe · enviar informe al cliente · crear rama.
- **UL-2**: 1 ítem por módulo (1:1) · CI/CD verde · code review (TL).
- **UL-3**: upgrade sin errores · scripts pre/post-migrate · validación técnica y de datos · entrega al cliente (código+BD o rama Odoo.sh).
- **UL-4**: triar feedback · corregir incidencias · re-validar CI · acordar fecha de producción · re-migración final · entrega final / rama Odoo.sh.

Algunos ítems (logs sin tracebacks, módulos instalados, conteos de registros, pipeline CI) los puede verificar y marcar Claude vía API tras crear las tareas; el resto se marcan manualmente.

### Gates (entregables)

- **Gate A — Informe de módulos enviado** (fin UL-1): inventario 1:1, estimación, informe enviado, rama creada.
- **Gate B — Entrega para validación** (fin UL-3): módulos migrados, CI verde, BD sin errores, validación técnica/datos, entrega realizada.
- **Gate C — Cierre y puesta en producción** (fin UL-4): feedback triado, incidencias corregidas, fecha de producción acordada, re-migración final ejecutada, entrega final.

---

## Fechas (compute_ul_phase_dates)

Pesos del rango `date_start → date_golive` (= fecha de producción final):
UL-1 15% · UL-2 45% · UL-3 25% · UL-4 15%. **UL-4 termina exactamente en `date_golive`.**
Cada gate es 1 día al final de su fase. **No hay soporte post-golive** (a diferencia del interno).

Sin mínimo de 6 semanas: basta `date_golive > date_start`.

---

## Comando exacto del motor

```bash
python3 <ruta migration-assistant>/scripts/create_migration_tasks.py \
    --analysis /tmp/inventario_migracion.md \
    --url "$ODOO_URL" --db "$ODOO_DB" --user "$ODOO_USER" --key "$ODOO_KEY" \
    --project-id <ID> --version 19.0 \
    --from-version <X> --to-version <Y> \
    --client-name "<Cliente>" \
    --profile upgrade-lab \
    --project-size <small|medium|large> \
    --date-start <YYYY-MM-DD> --date-golive <YYYY-MM-DD> \
    --analysis-hours <h> --db-hours <h> --fix-hours <h> \
    --with-checklists --with-gates
```

Notas:
- `--profile upgrade-lab` fuerza `strategy=1to1` internamente; no pasar `--strategy`.
- NO pasar `--infra-hours --validation-hours --golive-hours --support-hours --studio-hours` (se ignoran en este carril).
- Las horas de UL-2 (código) son la suma del inventario, no se pasan por flag.
- `--with-gates` crea A/B/C; sin él no se crean gates. La gestión va por defecto (solo "Gestión del proyecto").
- Hosting: solo relevante si Odoo.sh (afecta al texto de "rama desplegada"). Por defecto `cloud-indaws`.

---

## Knowledge tree (reducido)

En Upgrade Lab el árbol Knowledge es más corto que el interno. Raíz + 4 hijos:

1. **Análisis e informe de módulos** (qué se migra / qué no)
2. **Scripts de migración** (pre/post-migrate versionados)
3. **Entregas y feedback del cliente** (log de iteraciones de entrega + incidencias)
4. **Cierre y puesta en producción** (re-migración final + fecha)

No se crean los artículos de Ensayo/Rehearsal, Infra ni Retrospectiva formal del carril interno.

---

## Validación post-creación

1. Nº de tareas creadas = **8** (4 MIG + 3 GATE + 1 gestión).
2. NO existe ninguna tarea `[INFRA]`, `[MIG] Testing funcional`, `[MIG] Puesta en producción (Go-Live)` ni `[MIG] Soporte`.
3. NO existe la tarea "Reuniones".
4. Las 3 gates son A/B/C (no Gate 0..7).
5. UL-4 y Gate C terminan en `date_golive`.
6. Tag Migration en las 4 `[MIG]` y en las 3 `[GATE]`; ausente en "Gestión del proyecto".
