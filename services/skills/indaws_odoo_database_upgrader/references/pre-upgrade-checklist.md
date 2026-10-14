# Pre-Upgrade Checklist

Checklist paso a paso para preparar una BD antes de enviarla al Odoo Upgrade Service.

## Checklist obligatorio

### 1. Informacion basica

- [ ] **Version origen confirmada** (`ir_config_parameter.database.version`)
- [ ] **Version destino definida** (ej: 18.0)
- [ ] **Edicion confirmada** (CE o EE)
- [ ] **Contrato Enterprise valido** (si destino es EE)
- [ ] **Tamano BD medido** (`pg_database_size()`)
- [ ] **Numero de registros principales contados** (partners, facturas, asientos)

### 2. Modulos

- [ ] **Lista de modulos instalados exportada** (nombre, version, autor, categoria)
- [ ] **Modulos clasificados** en Core / OCA / Custom / Obsoleto
- [ ] **Modulos obsoletos identificados** (sin datos, sin uso)
- [ ] **Modulos con dependencias eliminadas detectados** (procurement, etc.)
- [ ] **Modulos sin uso desinstalados** (state = 'uninstalled')

### 3. Datos

- [ ] **Balance contable verificado** (debe = haber, descuadre = 0)
- [ ] **Logs del sistema limpiados** (ir_logging)
- [ ] **Sesiones expiradas limpiadas** (ir_sessions)
- [ ] **Auditlog antiguo limpiado** (> 2 anos si existe)
- [ ] **Queue jobs completados limpiados** (queue_job state=done)
- [ ] **Attachments huerfanos identificados** (opcional: limpiar)
- [ ] **Datos corruptos verificados** (NaN, fechas invalidas)

### 4. Dump

- [ ] **Formato correcto** (`pg_dump -Fc` o SQL plano)
- [ ] **Dump generado desde BD limpia** (post-limpieza)
- [ ] **Tamano del dump verificado** (< 5 GB recomendado)
- [ ] **Dump probado** (restaurable en otra BD de test)

### 5. Filestore

- [ ] **Filestore localizado** y copiado aparte
- [ ] **Tamano filestore medido**

## Checklist recomendado

### Optimizaciones adicionales

- [ ] **VACUUM FULL ejecutado** (reduce tamano BD)
- [ ] **ANALYZE ejecutado** (actualiza estadisticas)
- [ ] **Notificaciones mail antiguas limpiadas** (> 3 anos, tipo notification)
- [ ] **Crons con nextcall antiguo actualizados**
- [ ] **Fetchmail desactivado** (evitar que intente conectar durante upgrade)

### Documentacion

- [ ] **Snapshot del conteo de registros guardado** (para comparar post-upgrade)
- [ ] **Balance contable guardado** (total debe, total haber)
- [ ] **Lista de tablas OCA/custom guardada** (para verificar que sobreviven)
- [ ] **Token de upgrade guardado** (del output del CLI o `upgrade.log`)

## Queries de verificacion rapida

```sql
-- Script todo-en-uno: ejecutar contra la BD limpia antes de hacer el dump

DO $$
DECLARE
    v_version text;
    v_size text;
    v_modules int;
    v_partners int;
    v_invoices int;
    v_balance numeric;
BEGIN
    -- Version
    SELECT value INTO v_version FROM ir_config_parameter
    WHERE key = 'database.version';
    RAISE NOTICE 'Version: %', v_version;

    -- Tamano
    SELECT pg_size_pretty(pg_database_size(current_database())) INTO v_size;
    RAISE NOTICE 'Tamano BD: %', v_size;

    -- Modulos instalados
    SELECT count(*) INTO v_modules FROM ir_module_module
    WHERE state = 'installed';
    RAISE NOTICE 'Modulos instalados: %', v_modules;

    -- Partners
    SELECT count(*) INTO v_partners FROM res_partner;
    RAISE NOTICE 'Partners: %', v_partners;

    -- Facturas (adaptar segun version)
    BEGIN
        EXECUTE 'SELECT count(*) FROM account_invoice' INTO v_invoices;
        RAISE NOTICE 'Facturas (account_invoice): %', v_invoices;
    EXCEPTION WHEN undefined_table THEN
        EXECUTE 'SELECT count(*) FROM account_move WHERE move_type IN (''out_invoice'',''out_refund'',''in_invoice'',''in_refund'')' INTO v_invoices;
        RAISE NOTICE 'Facturas (account_move): %', v_invoices;
    END;

    -- Balance
    SELECT COALESCE(SUM(debit - credit), 0) INTO v_balance
    FROM account_move_line;
    RAISE NOTICE 'Balance (debe-haber): %', v_balance;

    IF v_balance != 0 THEN
        RAISE WARNING '*** BALANCE DESCUADRADO: % ***', v_balance;
    END IF;
END $$;
```

## Orden de ejecucion

```
1. Restaurar dump en BD temporal
2. Ejecutar queries de analisis (Fase 1)
3. Clasificar modulos
4. Desinstalar modulos obsoletos (UPDATE state = 'uninstalled')
5. Limpiar datos (DELETE logs, sesiones, etc.)
6. VACUUM FULL + ANALYZE
7. Verificar balance contable
8. Generar dump limpio (pg_dump -Fc)
9. Verificar que el dump es restaurable
10. Enviar al Upgrade Service
```
