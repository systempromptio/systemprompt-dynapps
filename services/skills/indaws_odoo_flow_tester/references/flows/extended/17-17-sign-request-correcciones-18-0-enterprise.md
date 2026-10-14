## 17. sign.request — Correcciones 18.0 (Enterprise)

### Correcciones validadas en tests en vivo (2026-03-10)

#### Requisitos de `request_item_id`
- Se debe proporcionar **un `request_item_id` por cada rol del template** (sign.template).
- Para solicitudes compartidas (shared), NO establecer `partner_id` en el request item.
- Usar contexto `{'no_sign_mail': True}` para prevenir el envio de emails al crear la solicitud.

**Ejemplo de creacion:**
```json
{
  "template_id": "<int>",
  "request_item_ids": [[0, 0, {"role_id": "<int>", "partner_id": "<int>"}]],
  "reference": "Documento test"
}
```
Con contexto: `{"no_sign_mail": true}`

---

