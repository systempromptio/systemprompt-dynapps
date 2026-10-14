# Workflows — permissions (JIT)

The tool reference is `../access-jit.md`. These are the sequences, including the one that
matters most: what to say when the answer is "you cannot, and here is who can".

---

## 1. A call was denied — what to do about it

Read `error_code` first. It tells you which of four situations you are in, and three of
them have different actions:

```text
orm_write { ... }   → { ok: false, error_code: "E_CATEGORY_NEEDS_GRANT",
                        request_hint: { scope: "category", category: "data_write",
                                        duration_hours: 2, environment_id: 4021 } }
```

| Code | Action |
| --- | --- |
| `E_CATEGORY_NEEDS_GRANT` | Request it with the **`request_hint` verbatim**, then retry once |
| `E_SCOPE_NOT_REQUESTABLE` | Already open. Just call the tool — no request needed |
| `E_CATEGORY_FORBIDDEN` / `E_SCOPE_FORBIDDEN` | **Never grantable here. Stop** and say so |
| `E_MISSING_LOCATION_ARG` | Not permissions at all — you omitted one of the five args |

**Never retry a denied call unchanged.** The denial is deterministic; the second attempt
fails identically and only adds noise to the audit log.

---

## 2. Ask for access and use it

```text
request_mcp_access {
  environment_id: 4021,
  scope: "category", category: "module_admin",
  duration_hours: 2,
  reason: "Install sale_management on acme-erp/main to reproduce the customer's pricing issue."
}
  → { grant_id: 5, name: "BKRG/2026/00500",
      state: "active", effective_state: "active",
      remaining_minutes: 120, auto_approved: true }

admin_install_module { ...five args..., module_names: ["sale_management"] }
```

- **The `reason` is stored and read by humans.** Write the sentence you would want to find
  if you were auditing this in six months.
- Ask for the category the **failed call** used, not the one that sounds right.
- `duration_hours` is clamped to the stage maximum. Long operations deserve a look at
  `remaining_minutes` before starting — beginning a 20-minute install with 4 minutes of
  grant left is a wasted attempt.

When you finish sensitive work on production, `revoke_mcp_access` closes the window early
and leaves a cleaner trail than letting it lapse.

---

## 3. The grant is pending

If you do not hold `mcp_approver` on that project, the grant is created **`pending`** and
nothing is unlocked until a person approves it.

```text
get_my_mcp_access { environment_id: 4021 }
  → { state: "pending", effective_state: "pending", name: "BKRG/2026/00700", … }
```

**Do not loop retrying the original call.** Poll `get_my_mcp_access` if you are waiting, or
— better — tell the user and stop.

What you say depends on who they are:

**Technical profile.** Give them the facts and let them chase it:

> "El permiso `module_admin` sobre *acme-erp/main* está pendiente de aprobación
> (`BKRG/2026/00700`). Necesita que lo apruebe alguien con rol `mcp_approver` en ese
> proyecto."

**Consulting profile.** No jargon, and hand them something they can forward:

> "He pedido permiso para instalar módulos en el entorno *main* de *acme-erp*, y está
> esperando la aprobación del equipo técnico. Cuando lo aprueben, seguimos.
> Si quieres darles prisa, esto es lo que necesitan saber:
> **entorno:** acme-erp / main · **permiso:** administración de módulos ·
> **referencia:** BKRG/2026/00700 · **motivo:** instalar sale_management para reproducir la
> incidencia de precios."

The point of the second version is that it is **forwardable**. A consultant should never
have to translate your explanation before asking for help.

---

## 4. See real values behind the masking

Some fields come back masked by stage policy:

```json
{ "id": 15, "name": "Aurora Belmonte", "email": "hash-c4e29035",
  "phone": "+34 ***1001", "_masked_fields": ["email", "phone"] }
```

That is not corrupt data and not a bug. It also blocks aggregating over those fields, on
purpose.

If the real value is genuinely necessary — you are debugging a delivery failure and need
the actual address — request the separate scope:

```text
request_mcp_access { environment_id: ..., scope: "unmask", duration_hours: 1,
                     reason: "Verify the delivery address of order S00123 after a failed shipment." }
```

**Ask for the shortest window that does the job**, and prefer not asking at all: most
questions ("does this contact have an email?", "how many have no phone?") are answerable
with the mask in place. Lifting PII protection is a decision you should be able to defend
by the sentence you wrote in `reason`.
