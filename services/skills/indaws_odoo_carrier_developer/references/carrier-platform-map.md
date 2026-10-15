# Spanish carrier platform map

Many ES carriers share the SAME backend platform. Identify the platform first —
it decides transport, auth, docIn/response shape and the known quirks, and lets
you clone an existing indaws_delivery_* module instead of starting cold.

Verified live against real endpoints 2026-07-13; the WSintegracion and GTS/Alertran rows re-verified 2026-08-19 against the providers' own documentation and DNS.

| Platform | Carriers (indaws_delivery_*) | Transport / auth | docIn & response | Verified quirks |
|----------|------------------------------|------------------|------------------|-----------------|
| **WSintegracion / asmred / geslotrans** | `gls_asm` (wsclientes.asmred.com), `transld` (transld.geslotrans.com:8443) | raw SOAP 1.2 over `requests`; `uidcliente` in the docIn | `docIn` embedded **RAW** (escaped ⇒ **HTTP 500** `ReadXmlDocument`); result comes as **child nodes** (or escaped text on some ops) — parse both; errors as `<RespuestaError><CodigoError><MensajeError>` | cancel op **differs per carrier**: `Anula` (asmred) vs `BorrarEnvio` (TRANSLD). TRANSLD has no `GetManifiesto` (build manifest from pickings). Demo UID: asmred `6BAB7A53-…` authenticates writes; TRANSLD v14 UID is dead ("Cliente no registrado"). **Label op: TRANSLD needs `EtiquetaEnvioTGL(codenvio, tipoEtiqueta, uidcliente)`** — the plain `EtiquetaEnvio` also exists in the WSDL, takes no uid and answers an EMPTY body. **Success is `<Resultado return="0"/>` and nothing else** (provider sheet: *"0 (zero): Shipment inserted Ok. Any other value: Error on insertion"*) — note this is the OPPOSITE of SAGEC below, where `0` means error. |
| **GTS / Alertran / Padua** | `ontime` (**prod `ontimegts.alertran.net`**, pre `gtsontimepre.alertran.net`), `enlaza` (base_url per client) | REST/JSON; **token** + relogin | **The body is a JSON ARRAY**, one element per shipment: `[{"respuestaDocuemtarEnvio": {...}}]`. Unwrap the array ONCE at the transport layer, then `.get(key)`; unwrapping the inner value instead ⇒ `AttributeError: 'list' object has no attribute 'get'` on every real call | retry on `codigo_error == "1"` (session expired). enlaza handles dict+list auth-error. Response keys carry typos the provider confirms as intentional (`respuestaDocuemtarEnvio`). **The prod host is NOT in the documentation** — the 14 official docs show only the pre one, and `gtsontime.alertran.net` (shipped for months) is NXDOMAIN. `detalleExpediciones` pages at 150 records via the `PAGINA` tag (doc v3.1). |
| **SAGEC** | `mrw` (sagec.mrw.es, `-test` env) | raw SOAP 1.1; **AuthInfo SOAP header** (CodigoFranquicia/Abonado/Departamento/UserName/Password) | structured `<mrw:...>` fields (no docIn blob) | **`Estado == 0` means ERROR**, non-zero = OK (`bool(int(estado))`) — verified: auth error returns `Estado=0`. Tracking is a *separate* service (`wssgmntnvs.asmx`) with auth in the body and `Estado` "true"/"1". |
| **SEUR ATLAS** | `seur` (servicios.api.seur.io, `apipre` test) | REST/JSON; **OAuth2** `grant_type=password` at `/pic_token`, Bearer | JSON; some endpoints wrap under `data`, others not — check per endpoint | has a real rate API (`/pic/v1/valuate`). Bad creds ⇒ 401 `invalid_client`. **Must retry once on 401.** |
| **Correos Express** | `correos_express` (cexpr.es, `test.cexpr.es`) | REST/JSON POST; **HTTP Basic Auth** | JSON; business errors in `codigoRetorno`/`codErr` in a 200 body | **labels are DOUBLE base64** (`b64decode(b64decode(...))`). |
| **MRW SAGEC / others** | — | — | — | Add rows as new platforms are integrated. |

## How to use

1. Given a new carrier, find its platform (endpoint domain is the tell:
   `asmred`/`geslotrans` → WSintegracion; `alertran` → GTS; `sagec` → SAGEC; `seur.io` → ATLAS; `cexpr` → Correos Express).
2. Clone the sibling module for that platform; swap endpoint/namespace/service codes/quirks.
3. **Do NOT assume the source module's operations exist for your carrier** — verify each op
   against the actual WSDL/`?op=` page (e.g. TRANSLD lacked `Anula`/`GetManifiesto` that the
   asmred sibling used). A blind port carries latent, silent bugs.
4. **Green mocked tests do NOT prove the wire format** — run a read-only live smoke test
   (see common-pitfalls: "Verify the wire format live").
