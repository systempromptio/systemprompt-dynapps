# Handover — Paquete de entrega

Alcance de Indaws en la migración (de DESIGN.md): **migración de módulos + BD +
correcciones**. NO testing funcional, NO levantar entorno de test (lo monta
Dynapps). La entrega es **código en GitLab + dump migrado**.

## Pre-requisito
- `quality-gate` en **verde** sobre el código migrado. Sin él, no se hace handover.

## Checklist de entrega
- [ ] **Código en GitLab**: rama/tag de la versión destino (≈19.0) pusheada, sin
      cambios sin commitear. Trazabilidad git completa (Dynapps trabaja con git, no
      Studio).
- [ ] **Dump migrado** entregado a Dynapps (Indaws no lo restaura ni hostea).
- [ ] **Inventario de módulos migrados**: lista 🔴 Custom migrados, OCA/nativos
      excluidos o sustituidos por port v19, con su versión final.
- [ ] **Notas de migración**: cambios de API relevantes, módulos que requirieron
      retrabajo, módulos que superaron el cap ~20h (con justificación).
- [ ] **Correcciones pendientes**: errores reportados por Dynapps ya resueltos o
      listados para soporte.
- [ ] **Go-live / config a producción**: delegar a `odoo-copy-staging-master`.
      Este skill no copia configuración; solo verifica que se hizo.
- [ ] **Aceptación del cliente**: documento generado por
      `project-validation-drafting` y referenciado en el cierre.

## Qué NO entra en el handover
- Testing funcional ni evidencias de test (responsabilidad de Dynapps).
- Entorno de test/staging levantado (lo monta Dynapps).
- Envío por correo del paquete desde Odoo (regla dura; el envío es manual).

## Salida
Resumen de handover (markdown) con los enlaces a GitLab, referencia al dump
entregado, inventario de módulos y la referencia al documento de aceptación.
