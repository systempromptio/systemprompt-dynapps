# Ejemplo Senior — Bridge SII para reporting fiscal en tiempo real

**Titulo**: Bridge SII multi-motor para reporting fiscal AEAT | **Odoo**: 18.0
**Asignado**: Alberto Ruiz (Senior) | **Estimacion**: ~120h | **Limite**: Q2 2026

## Contexto de Negocio

La AEAT requiere reporting SII en tiempo real. La implementacion actual cubre casos basicos pero le faltan:
- Soporte multi-actividad
- Cadenas de facturas rectificativas
- Retry automatico ante caidas de AEAT

3 clientes bloqueados para go-live en v18.

## Objetivo

Un bridge que funcione con AMBOS motores fiscales (Enterprise nativo + OCA) para envio SII completo, incluyendo rectificativas y retry. Modulos separados (bridge pattern).

## Constraints

- Compatible con Enterprise Y Community (dos motores fiscales)
- No romper funcionalidad existente de `l10n_es_edi_sii`
- Modulos separados (bridge pattern, no monolito)
- Performance: batch de hasta 500 facturas
- Deadline compliance: Q2 2026

## Entregables

1. **Documento de diseno tecnico** — antes de codificar
2. Modulo(s) con tests
3. Path de migracion para clientes existentes
4. UAT con cliente Pharma

## Stakeholders

- David (CTO) para review de arquitectura
- Maria (contabilidad) para validacion de negocio
- Cliente Pharma para UAT

## Criterios de Aceptacion

- [ ] Documento de diseno tecnico aprobado por CTO antes de empezar codigo
- [ ] Modulo bridge funciona con motor Enterprise (`l10n_es_edi_sii`)
- [ ] Modulo bridge funciona con motor OCA (`l10n_es_facturae` + extensiones)
- [ ] Cadenas de rectificativas: factura A → B (rectifica A) → C (rectifica B) registradas correctamente en AEAT
- [ ] Retry automatico tras caida AEAT (con backoff exponencial)
- [ ] Test de carga: 500 facturas en batch sin timeout
- [ ] Path de migracion documentado para clientes en produccion
- [ ] UAT firmado por cliente Pharma

## Restricciones

- No tocar `l10n_es_edi_sii` directamente (cliente puede tener parches custom)
- Bridge pattern: modulo core + bridges (uno por motor)
- Performance: medir tiempo de envio SII por factura (objetivo <2s)

## Ayuda

- Sync de diseno: cuando tengas la propuesta tecnica
- Contacto: Toni por Slack o call directo
- David disponible para review arquitectura

## Entregables

1. [ ] Doc de diseno (Markdown + diagramas)
2. [ ] Modulo(s) con tests
3. [ ] Migration path documentado
4. [ ] UAT firmado por Pharma
