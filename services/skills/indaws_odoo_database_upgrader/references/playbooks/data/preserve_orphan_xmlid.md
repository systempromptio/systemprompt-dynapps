# Playbook: Preservar xml_id huérfano

Registro físico vivo en la BD con FK a otras tablas, pero su xml_id apunta a
un módulo que ya no existe. El upgrade intentará borrarlo y romperá las FKs.

## Requisitos del contexto
- `to_version`: cualquiera

## Aplicabilidad
- Existe fila en `ir_model_data` con `module = '<modulo_no_instalado>'`
- El registro físico (`res_id` en `model`) tiene foreign keys vivas

## Flujo
1. Identificar registros con xml_id de módulos no instalados
2. Para cada uno: rebindar el xml_id a un módulo "preservación" (ej.
   `_preserve` o el módulo que más sentido tenga)
3. El registro físico queda intacto; el upgrade ya no lo borra

## Helpers a invocar
```yaml
- id: preserve_fiscalyear         # ejemplo concreto, params por defecto
  type: indaws_migration_kit.helpers.odoo.preserve_xmlid.PreserveOrphanXmlid
  params:
    module: account_fiscal_year
    name: fiscalyear
    model: date.range.type
```

## Pitfalls
- Verificar que el registro físico realmente está vivo (`fetchone()` > 0) antes
  de re-asignar
- Si el módulo destino del rebind también desaparece más adelante, queda nuevamente huérfano
