# Secure Coding — Gates WARNING (8-12)

These gates don't block delivery but MUST be flagged in the output.

| Nivel | Efecto | Significado |
|-------|--------|-------------|
| **WARNING** | Entregar con aviso | Mejora recomendada, no bloquea pero se debe justificar |

---

## GATE 8: Rendimiento como Seguridad [WARNING]
**ISO A.8.6 — Gestion de capacidad**

### N+1 queries

```python
# WARNING — DoS involuntario
for record in self.search([]):
    partner_name = record.partner_id.name  # 1 query por record

# MEJOR — prefetch o batch
records = self.search([])
records.mapped('partner_id.country_id')  # prefetch en 2 queries
for record in records:
    partner_name = record.partner_id.name  # desde cache
```

### Search sin limite

```python
# WARNING — puede devolver millones de registros
records = self.env['stock.move'].search([])

# MEJOR — siempre limit en contextos web
records = self.env['stock.move'].search(domain, limit=1000)
```

### Endpoints publicos sin proteccion

```python
# WARNING — endpoint caro sin rate limit
@http.route('/api/heavy_report', auth='public')
def heavy_report(self):
    return self.env['account.move'].search_read([])

# MEJOR — limit + paginacion + auth
@http.route('/api/report', auth='user', methods=['POST'])
def report(self, page=1, limit=80):
    offset = (int(page) - 1) * int(limit)
    return self.env['account.move'].search_read([], limit=int(limit), offset=offset)
```

---

## GATE 9: Manejo de Errores [WARNING]
**ISO A.8.28**

### Stack traces expuestos

```python
# WARNING — exception raw al usuario
except Exception as e:
    return str(e)  # puede exponer SQL, paths internos

# MEJOR
except UserError:
    raise  # UserError esta pensado para el usuario
except Exception:
    _logger.exception("Error processing request")
    raise UserError("Error interno. Contacte al administrador.")
```

### Excepciones silenciadas

```python
# WARNING — error tragado
try:
    record.action_confirm()
except Exception:
    pass  # imposible diagnosticar

# MEJOR
except Exception:
    _logger.exception("Failed to confirm %s", record.name)
    raise
```

---

## GATE 10: Integraciones Externas [WARNING]
**ISO A.8.26**

### Timeouts obligatorios

Pylint_odoo enforza esto via la regla `external-request-timeout` (activa en el `.pylintrc-mandatory` del skeleton). `requests.get/post/put/patch/delete` y `urllib` sin `timeout=` cuelgan el worker indefinidamente si el endpoint remoto no responde — DoS involuntario.

```python
# WARNING
response = requests.get('https://api.external.com/data')

# MEJOR
response = requests.get('https://api.external.com/data', timeout=30)
```

### Verificacion SSL

```python
# WARNING
response = requests.get(url, verify=False)

# MEJOR
response = requests.get(url, verify=True)
```

### Secrets en URLs

```python
# WARNING — API key en URL (queda en logs)
url = f"https://api.example.com/data?api_key={api_key}"

# MEJOR — en headers
response = requests.get(
    'https://api.example.com/data',
    headers={'Authorization': f'Bearer {api_key}'},
    timeout=30,
)
```

---

## GATE 11: Concurrencia y Transacciones [WARNING]
**ISO A.8.28**

### Race conditions en cantidades

```python
# WARNING — lectura-escritura sin lock
qty = record.available_qty
if qty >= requested:
    record.write({'available_qty': qty - requested})

# MEJOR — FOR UPDATE lock
cr.execute("SELECT available_qty FROM stock_quant WHERE id = %s FOR UPDATE", [record.id])
qty = cr.fetchone()[0]
if qty >= requested:
    cr.execute("UPDATE stock_quant SET available_qty = %s WHERE id = %s",
               [qty - requested, record.id])
```

### Idempotencia en webhooks

```python
# WARNING — webhook procesado multiples veces
order.action_confirm()  # se ejecuta cada vez que llega

# MEJOR — idempotente
if order.state == 'draft':
    order.action_confirm()
```

---

## GATE 12: Compliance Regulatorio [WARNING]
**ISO A.5.34 / RGPD**

- Datos personales con mecanismo de export/borrado RGPD
- Heredar `mail.thread` para trazabilidad
- Cron de limpieza para datos con retencion limitada
- Recopilar datos personales con consentimiento registrado (timestamp + origen)

---

## RESUMEN: Matriz Completa de Gates

| Gate | Control ISO | Que valida | Nivel |
|------|-----------|------------|-------|
| **1. Inyeccion** | A.8.28 | SQL params, safe_eval, html_sanitize, Markup | **BLOQUEANTE** |
| **2. Acceso** | A.8.3 | ACLs, record rules, groups, check_company, sudo | **BLOQUEANTE** |
| **3. RPC** | A.8.26 | Metodos privados, auth controller, CSRF | **BLOQUEANTE** |
| **4. Datos sensibles** | A.5.12/RGPD | No loguear secrets, campos restringidos, no hardcode | **BLOQUEANTE** |
| **5. Dependencias** | A.8.8/A.8.9 | CVEs, config externalizada, no debug en prod | **BLOQUEANTE** |
| **6. Auditabilidad** | A.8.15 | mail.thread, tracking, message_post, no borrar logs | **BLOQUEANTE** |
| **7. Server actions** | A.8.28 | No commit, no destructivo, sudo justificado | **BLOQUEANTE** |
| **8. Rendimiento** | A.8.6 | N+1, search sin limit, endpoints sin proteccion | **WARNING** |
| **9. Errores** | A.8.28+ | Stack traces, excepciones silenciadas | **WARNING** |
| **10. Integraciones** | A.8.26+ | Timeout, SSL verify, secrets en headers | **WARNING** |
| **11. Concurrencia** | A.8.28+ | Race conditions, idempotencia webhooks | **WARNING** |
| **12. Compliance** | A.5.34 | RGPD export/borrado, retencion, consentimiento | **WARNING** |

## Uso desde otros skills

Cualquier skill que genere codigo Odoo debe:
1. Leer `secure-coding-blocking.md` en el STEP de validacion
2. Pasar los 7 gates BLOQUEANTES antes de entregar
3. Leer `secure-coding-warning.md` para flaggear warnings
4. Si falla un gate BLOQUEANTE, corregir antes de entregar
