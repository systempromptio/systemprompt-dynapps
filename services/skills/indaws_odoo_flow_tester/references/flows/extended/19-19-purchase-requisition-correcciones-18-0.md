## 19. purchase.requisition — Correcciones 18.0

### Correcciones validadas en tests en vivo (2026-03-10)

#### Metodo correcto: `action_confirm` (NO `action_in_progress`)
- El metodo para confirmar una requisicion es `action_confirm`, **NO** `action_in_progress`.
- Estados: `draft` → `confirmed` → `done`
- Para blanket orders (ordenes abiertas), las lineas requieren `price_unit` obligatorio.

**Flujo correcto:**
```
draft → action_confirm() → confirmed → action_done() → done
```

---

