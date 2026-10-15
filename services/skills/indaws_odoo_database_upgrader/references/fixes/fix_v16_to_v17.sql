-- =============================================================================
-- FIXES: Transicion v16 → v17
-- Aplicar sobre la BD ANTES de enviar al Odoo Upgrade Service
-- Todos los fixes son idempotentes (seguros si se ejecutan varias veces)
-- =============================================================================


-- -----------------------------------------------------------------------------
-- FIX-002: Dobles llaves en atributo context de vistas XML
-- -----------------------------------------------------------------------------
-- Categoria: OCA-CONDITIONAL (partner_contact_in_several_companies)
-- Error    : SyntaxError: unmatched '}' in context attribute (attr_domains2expr)
-- Script   : base/17.0.1.3/attr_domains2expr.py → ast.parse() sobre atributos
-- Causa    : Vistas OCA con '}}' en atributos context/attrs/domain que Odoo
--            toleraba en v10-v16 pero que v17 valida con ast.parse().
-- Condicion: Si algun modulo OCA introdujo '}}' en vistas XML.
--            Modulo conocido: partner_contact_in_several_companies
-- Gotcha   : Requiere sintaxis E-string de PostgreSQL para escapar '}}',
--            las comillas estandar no funcionan.
--            Falsos positivos: dicts anidados {'key': {'sub': True}} son validos,
--            no reemplazar sin validar con ast.parse() primero.
-- Fuente   : Tecniman v10→v18 (iteracion 2, 2025)
-- -----------------------------------------------------------------------------

-- PASO 1: Deteccion — ejecutar primero para identificar vistas candidatas
-- Devuelve las vistas con '}}' en arch_db para revision manual
SELECT
    id,
    name,
    model,
    -- Extraer fragmento con el contexto problematico
    substring(arch_db from position('}}' in arch_db) - 50 for 100) AS contexto
FROM ir_ui_view
WHERE arch_db LIKE '%}}%'
  AND active = true
ORDER BY id;

-- PASO 2: Fix — reemplazar '}}' por '}' solo en atributos context/attrs/domain
-- IMPORTANTE: Validar cada resultado del paso 1 con ast.parse() en Python antes
-- de aplicar. Un '}}' dentro de un dict anidado {"key": {"sub": 1}} es valido.
--
-- Patron de reemplazo (ajustar segun los resultados del paso 1):
-- UPDATE ir_ui_view
--     SET arch_db = REPLACE(arch_db, E'supplier}}\\"', E'supplier}\\"')
-- WHERE id = <ID_DE_LA_VISTA>;
--
-- Script Python de validacion:
-- import ast
-- for row in candidatas:
--     try:
--         ast.parse(row['contexto'], mode='eval')
--         print(f"Vista {row['id']}: contexto VALIDO (no aplicar fix)")
--     except SyntaxError:
--         print(f"Vista {row['id']}: contexto INVALIDO (aplicar fix)")


-- -----------------------------------------------------------------------------
-- FIX-003: Tipo de columna incorrecto en product_supplierinfo.discount
-- -----------------------------------------------------------------------------
-- Categoria: OCA-CONDITIONAL (product_supplierinfo_discount)
-- Error    : column "discount" of relation "product_supplierinfo" already exists
--            with type "numeric" (expected double precision / float8)
-- Script   : Odoo ORM column creation durante migracion
-- Causa    : El modulo OCA product_supplierinfo_discount creo la columna como
--            numeric. Odoo 17+ la espera como double precision (float8).
-- Condicion: Si product_supplierinfo_discount esta instalado
-- Fuente   : Tecniman v10→v18 (iteracion 2, 2025)
-- -----------------------------------------------------------------------------

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'product_supplierinfo'
          AND column_name = 'discount'
          AND data_type = 'numeric'
    ) THEN
        ALTER TABLE product_supplierinfo
            ALTER COLUMN discount TYPE double precision
            USING discount::double precision;
        RAISE NOTICE 'FIX-003: product_supplierinfo.discount convertido a double precision';
    ELSE
        RAISE NOTICE 'FIX-003: product_supplierinfo.discount no requiere conversion (omitido)';
    END IF;
END $$;
