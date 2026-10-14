# Invariantes de factura (RD 1619/2012) + EDI (SII / Veri*Factu / Facturae)

Cada invariante con cita. Verificar contra fuente oficial AEAT y contra la implementación real
(la clave de régimen puede NO salir de donde parece — ver gotcha).

## Factura conforme (RD 1619/2012, Reglamento de facturación)
- **Contenido obligatorio** (art. 6): número y serie, fecha expedición, fecha operación si distinta,
  NIF, datos de las partes, base, tipo, cuota, contraprestación.
- **Mención de régimen** (art. 6.1): p.ej. "régimen especial de las agencias de viajes",
  "régimen especial de bienes usados", "inversión del sujeto pasivo", "facturación por el
  destinatario", "exención" con el precepto. La ausencia de mención obligatoria = no conforme.
- **No desglose**: en regímenes que lo imponen (REAV art. 142 LIVA) no se consigna la cuota
  por separado. Verificar coherencia entre mención y desglose.
  - **El art. 142 gobierna el DOCUMENTO IMPRESO, no el asiento contable.** Un `account.move`
    con la cuota "comprendida" (correcta a nivel de línea) sigue siendo NO conforme si el
    **PDF/QWeb** que recibe el cliente imprime la cuota desglosada. La conformidad se juzga
    sobre el documento renderizado, no sobre `account.move.line`.
  - **Presunción de NO conforme si el módulo NO overridea el report de factura** en un régimen
    de margen (REAV/REBU): el `account.report_invoice_document` estándar SIEMPRE imprime el
    bloque `tax_totals` (base+cuota) y el importe por línea. Sin override propio del report, el
    documento desglosa la cuota y/o revela coste+margen al cliente → incumple art. 142 aunque
    el asiento sea correcto. Solo se levanta la presunción renderizando el report y comprobando
    que NO sale el desglose de cuota.
- **Rectificativas** (art. 15): serie propia, referencia a la factura rectificada, motivo.
- **Numeración**: correlativa por serie, sin huecos (verificar la secuencia del journal).

## SII (Suministro Inmediato de Información)
- **`ClaveRegimenEspecialOTrascendencia`**: una clave por documento; código correcto según la
  operación (FAQ SII AEAT, Libro de Facturas Expedidas/Recibidas). p.ej. `05` = agencias de viajes.
- **Gotcha (indawscore)**: la clave puede salir de `fiscal_position.l10n_es_sii_registration_key_sale`
  (o de la factura) y NO de `account.tax._l10n_es_get_regime_code`. **Verificar de dónde sale de
  verdad** en el stack concreto, no asumir el mecanismo estándar.
- Base y cuota del nodo SII deben cuadrar con la factura.

## Veri*Factu
- **Aplicabilidad del impuesto** (`l10n_es_applicability`): `01`=IVA, `02`=IPSI, `03`=IGIC; sin
  valor cae a `05` "Otros" y la clave de régimen específica **no se sugiere**.
- **Clave de régimen**: 2 dígitos (p.ej. `05`) entra en el bucket de aplicabilidad IVA `01`;
  una por documento.
- **Inmutabilidad / cadena de hash**: tras el registro, el documento Veri*Factu no debe poder
  modificarse; las correcciones van por anulación/rectificación encadenada. Un desarrollo que
  permita editar tras registro es NO conforme — verificar en runtime.

## Facturae
- Estructura y firma XAdES; régimen y datos AAPP correctos. (Alcance según proyecto.)

## Coherencia cruzada
- Base/cuota de factura ↔ nodo EDI ↔ modelos AEAT (303/347/349/390). Un desarrollo que altere
  cualquiera de los tres debe mantener los tres coherentes.

## Cómo verificar
Construir la factura del borde en runtime, postearla (o dejarla en borrador si el EDI se computa
en borrador) y asertar sobre el **DOCUMENTO RENDERIZADO** (no solo el asiento): mención presente,
desglose acorde al régimen (renderizar el report y comprobar que en un régimen sin-desglose NO
aparece la fila `tax_totals` de cuota), que el documento no revela coste/margen indebidamente,
clave de régimen resuelta al valor esperado, y (Veri*Factu) que no se puede modificar tras
registro. Ver `runtime-verification.md` (incluye cómo renderizar el report).
