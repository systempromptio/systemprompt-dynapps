# Playbook: Comparison BD nuestra vs referencia

Comparar la BD migrada por el kit contra una **referencia** (típicamente el dump
del cliente trabajado por staging propio del cliente o por un consultor externo)
para detectar olvidos.

## Requisitos del contexto
- `to_version`: cualquiera
- Existe dump de referencia (`*_trabajado.dump` o similar)

## Aplicabilidad
- Siempre que exista un dump de referencia legítimo

## Flujo
1. Restaurar el dump ref en el container analyzer:
   ```bash
   docker exec odoo-upgrade-analyzer-pg16 pg_restore -U odoo -d ref_db /tmp/ref.dump
   ```
2. Lanzar comparison (8 niveles):
   ```bash
   indaws-migrate compare \
       --pipeline migration.yaml \
       --ref-db ref_db \
       --output docs/comparison_v18_ref.md
   ```
3. Revisar críticos (módulos faltantes, campos custom no aplicados, vistas
   no migradas, etc.)
4. Cada crítico → playbook concreta (ej. añadir módulo missing a `modules:`)

## Niveles
1. Módulos (state, version)
2. Campos custom / Studio (ir_model_fields)
3. Vistas (ir_ui_view)
4. Server actions / automations / crons
5. Configuración contable (journals, taxes, accounts)
6. Permisos (groups, access, rules)
7. Customizaciones técnicas (`arch_db` snapshots, server actions)
8. Datos transaccionales (counts comparables)

## Helpers a invocar
- Subcomando `compare` del CLI

## Pitfalls
- La BD ref puede ser de versión distinta — la comparison normaliza pero algunos
  campos no son comparables directamente (`account.account.code` → `code_store` jsonb en v18)
- Datos transaccionales: counts comparables solo si freeze efectivo
