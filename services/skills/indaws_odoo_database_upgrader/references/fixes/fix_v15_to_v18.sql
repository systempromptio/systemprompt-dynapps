-- =============================================================================
-- FIXES: Transicion v15 → v18
-- Aplicar sobre la BD ANTES de enviar al Odoo Upgrade Service
-- Todos los fixes son idempotentes (seguros si se ejecutan varias veces)
-- =============================================================================


-- -----------------------------------------------------------------------------
-- FIX-004: Menus OCA huerfanos causan AccessError en TestCrawler
-- -----------------------------------------------------------------------------
-- Categoria: OCA-CONDITIONAL (sale_order_type)
-- Error    : FAIL: TestCrawler.test_check
--            AccessError: No puede ingresar a los registros 'sale.order.type.rule'
--            (sale.order.type.rule)
-- Script   : test_mock_crawl.py → mock_view_form → sale_order_type.sot_sale_order_type_menu
-- Causa    : El Upgrade Service ejecuta TestCrawler tras migrar a v18. Si un modulo
--            OCA (ej: sale_order_type) estaba instalado en v15 pero NO tiene port a v18,
--            sus menus siguen activos y apuntan a modelos sin ir.model.access definido
--            en la nueva version. El test falla con AccessError.
-- Condicion: Si sale_order_type esta instalado en origen y target >= v18
-- Patron   : Aplicable a CUALQUIER modulo OCA sin port que deje menus activos.
-- Fuente   : Pharmactive v15→v18 (iter_001, 2026)
-- -----------------------------------------------------------------------------

-- PASO 1: Desactivar menus del modulo sale_order_type
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM ir_module_module
        WHERE name = 'sale_order_type' AND state = 'installed'
    ) THEN
        UPDATE ir_ui_menu SET active = false
        WHERE id IN (
            SELECT res_id FROM ir_model_data
            WHERE module = 'sale_order_type'
            AND model = 'ir.ui.menu'
        );
        RAISE NOTICE 'FIX-004: Menus sale_order_type desactivados';

        -- Tambien desactivar menus de modulos que extienden sale_order_type
        UPDATE ir_ui_menu SET active = false
        WHERE id IN (
            SELECT res_id FROM ir_model_data
            WHERE module = 'sale_order_type_ext'
            AND model = 'ir.ui.menu'
        );
        RAISE NOTICE 'FIX-004: Menus sale_order_type_ext desactivados';
    ELSE
        RAISE NOTICE 'FIX-004: sale_order_type no instalado (omitido)';
    END IF;
END $$;


-- -----------------------------------------------------------------------------
-- FIX-005: Deteccion generica de menus OCA huerfanos
-- -----------------------------------------------------------------------------
-- Categoria: OCA-CONDITIONAL (deteccion generica — NO aplica cambios)
-- Error    : Variantes de FIX-004 con otros modulos OCA sin port
-- Script   : test_mock_crawl.py (TestCrawler del Upgrade Service)
-- Causa    : Modulos OCA sin port dejan menus activos que apuntan a modelos
--            sin ir.model.access en la version destino.
-- Condicion: Ejecutar SELECT para detectar. Solo aplicar UPDATE tras revision manual.
-- Modulos conocidos:
--   - sale_order_type (confirmado en Pharmactive)
--   - partner_contact_in_several_companies (variable segun OCA v18)
--   - sale_order_line_date (absorbido en core v17)
-- Fuente   : Patron preventivo basado en Pharmactive v15→v18
-- -----------------------------------------------------------------------------

-- DETECCION: Listar menus activos de modulos OCA conocidos sin port
-- Ejecutar este SELECT primero para detectar. Solo aplicar UPDATE si hay resultados.

-- SELECT
--     imd.module,
--     im.name AS menu_name,
--     im.active
-- FROM ir_ui_menu im
-- JOIN ir_model_data imd ON imd.res_id = im.id AND imd.model = 'ir.ui.menu'
-- WHERE imd.module IN (
--     -- Añadir aqui modulos OCA que causan TestCrawler failures
--     'sale_order_type',
--     'sale_order_type_ext'
--     -- 'otro_modulo_oca'
-- )
-- AND im.active = true
-- ORDER BY imd.module, im.name;


-- -----------------------------------------------------------------------------
-- FIX-006: Desactivar acciones de ventana de modulos OCA sin port
-- -----------------------------------------------------------------------------
-- Categoria: OCA-CONDITIONAL (sale_order_type)
-- Error    : Variante del FIX-004 pero con ir.actions.act_window en vez de menus.
--            Si un modulo OCA tiene acciones que apuntan a modelos sin access rules,
--            el TestCrawler tambien falla.
-- Script   : test_mock_crawl.py (TestCrawler del Upgrade Service)
-- Causa    : Acciones de ventana de modulos OCA sin port apuntan a modelos sin ACL
-- Condicion: Si sale_order_type esta instalado en origen y target >= v18
-- Fuente   : Patron preventivo basado en Pharmactive v15→v18
-- -----------------------------------------------------------------------------

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM ir_module_module
        WHERE name = 'sale_order_type' AND state = 'installed'
    ) THEN
        -- Desactivar acciones de ventana del modulo
        UPDATE ir_act_window SET active = false
        WHERE id IN (
            SELECT res_id FROM ir_model_data
            WHERE module IN ('sale_order_type', 'sale_order_type_ext')
            AND model = 'ir.actions.act_window'
        );
        RAISE NOTICE 'FIX-006: Acciones sale_order_type desactivadas';
    ELSE
        RAISE NOTICE 'FIX-006: sale_order_type no instalado (omitido)';
    END IF;
END $$;


-- -----------------------------------------------------------------------------
-- FIX-007: Limpiar rules de modulos OCA sin port
-- -----------------------------------------------------------------------------
-- Categoria: OCA-CONDITIONAL (sale_order_type)
-- Error    : ir.rule de modulos OCA provocan errores durante la migracion
--            por referencias a modelos que ya no existen en v18.
-- Script   : TestCrawler / ORM rule evaluation
-- Causa    : Rules activas que referencian modelos sin ACL en version destino
-- Condicion: Si sale_order_type esta instalado en origen y target >= v18
-- Fuente   : Patron preventivo basado en analisis Pharmactive v15→v18
-- -----------------------------------------------------------------------------

DO $$
BEGIN
    -- Desactivar rules de modulos OCA sin port conocidos
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'ir_rule' AND column_name = 'active'
    ) THEN
        UPDATE ir_rule SET active = false
        WHERE id IN (
            SELECT res_id FROM ir_model_data
            WHERE module IN ('sale_order_type', 'sale_order_type_ext')
            AND model = 'ir.rule'
        );
        RAISE NOTICE 'FIX-007: Rules sale_order_type desactivadas';
    END IF;
END $$;
