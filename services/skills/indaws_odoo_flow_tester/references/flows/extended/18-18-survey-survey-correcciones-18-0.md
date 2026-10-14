## 18. survey.survey — Correcciones 18.0

### Correcciones validadas en tests en vivo (2026-03-10)

#### NO tiene campo `state`
- `survey.survey` **NO tiene campo `state`** en Odoo 18.0.
- Usa unicamente un campo `active` (Boolean) para controlar visibilidad.
- Los metodos `action_open`, `action_close`, `action_draft` **NO EXISTEN**.
- Para archivar/desarchivar, usar los metodos estandar:
  - Archivar: `action_archive()`
  - Desarchivar: `action_unarchive()`

**Flujo correcto:**
```
active=True (visible) ←→ active=False (archivada)
    action_archive()  /  action_unarchive()
```

---

