## 16. mailing.mailing — Correcciones 18.0

### Correcciones validadas en tests en vivo (2026-03-10)

#### `action_draft` NO existe
- El metodo `action_draft` **NO EXISTE** en `mailing.mailing` en Odoo 18.0.
- `action_cancel` ya resetea el estado directamente a `'draft'` (no a un estado intermedio 'cancel').
- Para volver un mailing a borrador desde cualquier estado cancelable, usar unicamente `action_cancel`.

**Flujo correcto de estados:**
```
draft → in_queue → sending → done
         ↓
    action_cancel → draft (NO pasa por estado 'cancel')
```

---

