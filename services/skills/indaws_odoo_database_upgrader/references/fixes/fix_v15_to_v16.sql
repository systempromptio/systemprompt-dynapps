-- =============================================================================
-- FIXES: Transicion v15 → v16
-- Aplicar sobre la BD ANTES de enviar al Odoo Upgrade Service
-- Todos los fixes son idempotentes (seguros si se ejecutan varias veces)
-- =============================================================================


-- -----------------------------------------------------------------------------
-- FIX-001: Columna website_id ausente en ir_ui_view
-- -----------------------------------------------------------------------------
-- Categoria: GENERIC
-- Error    : psycopg2.errors.UndefinedColumn: column u.website_id does not exist
-- Script   : portal/16.0.1.0/post-~reset-cowed-view.py
-- Causa    : El script de migracion del modulo portal asume que existe
--            ir_ui_view.website_id (creada por el modulo website).
--            Si website NO esta instalado, la columna no existe y falla.
-- Condicion: Siempre (bug en script base, inofensivo si website ya esta instalado)
-- Fuente   : Tecniman v10→v18 (iteracion 1, 2025)
-- -----------------------------------------------------------------------------

ALTER TABLE ir_ui_view ADD COLUMN IF NOT EXISTS website_id integer;
