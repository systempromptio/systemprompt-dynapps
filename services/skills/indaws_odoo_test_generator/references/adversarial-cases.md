# Adversarial & boundary cases (anti "test theater")

> Read **before delivering any test suite**. The #1 cause of bugs surviving a
> green suite is *happy-path tests over happy-path data*: the test confirms the
> case the author already imagined, and never probes the edge that breaks it.
> Two real bugs slipped past "green" suites this way (silent amount truncation;
> `€`→`EUR` breaking fixed-width alignment). Both were caught only when the test
> used adversarial input **and** asserted the exact invariant.

## The two golden rules (apply to EVERY test)

1. **The input must reach the output.** Before asserting on output, prove the
   value you control actually appears in it. A length/format assertion on output
   that never received your tricky value is *hollow* — it passes regardless.
   ```python
   # WRONG: partner.name never reaches a Tipo-0 A3 line → assertion is meaningless
   # RIGHT: assert the controlled text is present, THEN assert the invariant
   self.assertIn('Producto', text, "precondición: el texto entró (si no, test inútil)")
   self.assertEqual(len(line), 510)
   ```
2. **Assert the exact invariant, not a proxy.** `assertIn('123', out)`,
   `assertTrue(attachment)`, "no raise", "len > 0" are proxies. Assert the parsed
   value equals the expected value to the cent / the exact node / the exact state.
   ```python
   # WRONG (passed while truncating .99→.9): assertIn('12345678901', importe)
   # RIGHT: self.assertAlmostEqual(float(importe), 12345678901.99, places=2)
   ```

## Boundary checklist by data category

Generate at least one test per applicable row. Each is a class of latent bug.

| Category | Adversarial inputs to test | Invariant to assert |
|---|---|---|
| **Importes / decimales / impuestos** | overflow (valor > anchura de campo), negativo, cero, multi-impuesto (21%+10%), descuentos, multi-moneda | base+cuota == amount_tax/untaxed **al céntimo**; overflow **lanza** (no trunca); signo correcto |
| **Encoding / texto fijo** | `€` (expande a 'EUR'), `ñ á ç`, `½ Œ µ`, string que excede la longitud, vacío/False | longitud de registro EXACTA (chars **y** bytes); transliteración aplicada; sin desalineo |
| **Multi-company** | 2 compañías con mismos códigos; registro de A, proceso desde B | sin fuga cross-company (B no toca datos de A); `company_id` correcto |
| **Idempotencia / reintentos** | crear/procesar dos veces el mismo doc/clave; doble run del cron | no duplica; constraint único **dispara** (`assertRaises(IntegrityError)`); reintento reusa |
| **Seguridad (como usuario)** | `with_user(non_admin)` / `with_context(allowed_company_ids=...)` | acceso correcto; `sudo()` no filtra; record rules aplican. **NO testear solo como admin.** |
| **Estados / máquina** | handler que falla (1 de N), estado terminal ya fijado, transición inválida | error **aislado por registro**; estado no se pisa; OUT→queued / IN→done |
| **Fechas / tz** | naive vs aware, cambio de día por tz, formato | valor en tz esperada; sin shift de fecha |
| **Vacío / nulo** | partner sin vat/zip, factura sin líneas, campo False | emite blancos de longitud correcta, no `'False'`/`'None'` literal |

## How to wire it into generation

- In STEP 2 (generate), for each component, look up its categories above and emit
  the boundary tests, not only the happy path.
- In STEP 5 (validate, before delivering): re-read each generated test and ask
  *"would this pass even if the feature were broken?"* If yes, the assertion is a
  proxy — strengthen it per rule 2, and add the rule-1 precondition.
- Prefer a test that **fails first** (reveals the bug) over one written to pass.
  If a boundary test passes immediately, double-check the input actually exercised
  the edge (rule 1).
