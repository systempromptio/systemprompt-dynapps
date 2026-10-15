## 14. account.asset — Correcciones RPC (Enterprise)

### Correcciones validadas en tests en vivo (2026-03-10)

#### Metodo `pause(pause_date)` NO funciona via RPC
El metodo `pause(pause_date, message=None)` existe en el codigo fuente pero **no funciona via XML-RPC/JSON-RPC** porque el parametro `pause_date` (tipo `date`) no se auto-convierte correctamente al llamar desde RPC.

**Workaround:** Usar write directo para pausar:
```bash
update_record.sh account.asset '[<id>]' '{"state": "paused"}'
```

**Nota:** El metodo `set_to_paused` NO EXISTE. El metodo correcto es `pause(pause_date, message=None)`, pero solo funciona internamente en Odoo (no via RPC).

---

