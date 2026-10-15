## 5. Advertencias Fiscales — Secuencias y Legislacion

### 5.1 Cuando los huecos en secuencias son ilegales

En multiples jurisdicciones, la numeracion de facturas debe ser **correlativa y sin huecos**.
Un hueco implica que podria haberse emitido una factura que luego se oculto (evasion fiscal).

| Pais / Regulacion | Secuencia afectada | Nivel de exigencia | Consecuencia de huecos |
|--------------------|-------------------|-------------------|----------------------|
| **Espana — SII** | Facturas emitidas/recibidas | **Estricto** | El SII detecta huecos en la numeracion. Puede generar requerimiento de la AEAT. |
| **Espana — TicketBAI** | Facturas emitidas (Pais Vasco) | **Muy estricto** | Cada factura lleva hash encadenado + QR. Un hueco rompe la cadena y es detectable por Hacienda Foral. |
| **Espana — VeriFactu** | Facturas emitidas | **Muy estricto** | Sistema de verificacion de facturas. Encadenamiento criptografico. Huecos son anomalia fiscal. |
| **Francia — NF 525** | Facturas + tickets POS | **Estricto** | Certificacion obligatoria de software. Huecos en secuencia invalidan la certificacion. |
| **Italia — SDI** | Facturas electronicas | **Estricto** | Numeracion progresiva obligatoria por ley. |
| **Portugal** | Facturas con ATCUD | **Estricto** | Codigo unico + secuencia validada por Autoridade Tributaria. |
| **Mexico — CFDI** | Folios de factura | **Medio** | La numeracion interna es del contribuyente, pero el SAT puede cuestionar huecos significativos. |
| **Latinoamerica general** | Facturas legales | **Variable** | Muchos paises exigen numeracion correlativa (Argentina, Chile, Colombia, Peru). |

### 5.2 Cuando NO restaurar secuencias

Existen situaciones donde **restaurar la secuencia es mas peligroso que dejar el hueco**:

1. **Registros reales creados entre el inicio y fin del test**
   Si un usuario real creo una factura `INV/2026/0155` mientras el test creo
   `INV/2026/0153` y `INV/2026/0154`, restaurar a 153 causaria un **duplicado**.

2. **Facturas ya comunicadas a hacienda (SII/TicketBAI/VeriFactu)**
   Si una factura de test fue enviada a la AEAT antes de poder revertirla,
   restaurar la secuencia podria generar un numero duplicado que choque con
   el registro fiscal.

3. **Secuencias con hash encadenado**
   En journals con `restrict_mode_hash_table = True`, cada asiento tiene un hash
   que referencia al anterior. Eliminar un asiento intermedio y reusar su numero
   rompe la cadena de hashes.

4. **Entornos multi-usuario con alta concurrencia**
   Si hay multiples usuarios creando registros simultaneamente, la ventana entre
   "capturar snapshot" y "restaurar" puede contener registros legitimos.

### 5.3 Regla de oro

> **Solo restaurar secuencias si se puede garantizar al 100% que ningun registro
> real fue creado con un numero posterior al snapshot pre-test.**

---

