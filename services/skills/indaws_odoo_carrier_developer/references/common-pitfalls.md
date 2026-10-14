# Common Pitfalls

Comprehensive list of mistakes to avoid when creating delivery carrier modules.

---

## Architecture & Configuration

1. **Forgetting `ondelete` on selection_add:** Always define what happens when module is uninstalled
2. **Not using `sudo()` for credentials:** API keys are restricted to `base.group_system`
3. **Hardcoding URLs:** Use `prod_environment` field to switch between test/production URLs

## API Integration

4. **Not handling multi-package:** `send_shipping` receives pickings that may have multiple packages
5. **Blocking on API timeout:** Always set reasonable `timeout` on requests
6. **Not propagating tracking to related pickings:** See `send_to_shipper()` pattern for multi-step flows
7. **Assuming all carriers have cancel/rate APIs:** Some don't (e.g., Correos Express). Use fallback patterns.
8. **Not handling auth token expiry mid-operation:** Session tokens can expire between calls. Always implement retry logic in `_send_request()` -- detect auth errors in the response and retry once with a fresh token. See Auth Retry Pattern in [workflow-steps.md](workflow-steps.md).
9. **Using API version 1 when higher versions exist:** Some carriers (e.g., Enlaza) have versioned endpoints where higher versions return more detail. Always use the highest available API version (e.g., `VERSION: 6` for tracking).

## Labels & Logging

10. **Missing label prefix:** Use `self._get_delivery_label_prefix()` for label attachment names
11. **Not logging API calls:** Always use `self.log_xml()` when `debug_logging` is enabled
12. **Single base64 decode when carrier uses double:** Correos Express labels require double decode. Check the carrier API docs.

## State Tracking

13. **Not creating carrier states automatically:** When using `indaws_delivery_state_tracking`, states must be auto-created on carrier create/write.
14. **Only storing `carrier_tracking_ref`:** Some carriers return multiple references per shipment (expedition number, admission code, locator). Store the primary in `carrier_tracking_ref` and create extra `<provider>_*` fields on `stock.picking` for secondary references.

## Error Handling

15. **Generic error handler for all endpoints:** Carriers often use different error formats per endpoint (e.g., CEX: `codigoRetorno` for shipments, `codErr` for labels, `error` for tracking). Handle each separately.
16. **Assuming uniform response key casing:** Some carriers (e.g., Enlaza) use lowercase (`resultado`, `codigo_error`) in one endpoint and UPPERCASE (`RESULTADO`, `CODIGO_ERROR`) in another. Each request class method must parse its own response format.
17. **Putting all error codes in one dict:** Carriers with many endpoints often have different error code spaces per endpoint (e.g., Enlaza: ~220 codes across 8 dicts). Use separate dicts per endpoint with a cascading lookup helper.

## Version-Specific

18. **Using `numbercall` in cron XML:** Removed in Odoo 18 (**0 ocurrencias** en CE18 y CE19; existía en v16 `odoo/addons/base/models/ir_cron.py:68`, `ir_cron.ir_actions_server_id` y v17 `:67`).

> **`numbercall` no desapareció «sin más»: lo sustituyen DOS mecanismos** (medido 2026-08-17 contra
> CE18 `@99f5a33c64` y CE19 `@d1d161033`). Decir *"repiten indefinidamente por defecto"* es falso en
> los dos sentidos.
>
> 1. **Auto-desactivación por fallos repetidos — nueva en v18, no existía con `numbercall`.** Odoo
>    pone `active = False` cuando se cumplen **las dos** condiciones: `failure_count >=
>    MIN_FAILURE_COUNT_BEFORE_DEACTIVATION` (**5**) **y** han pasado más de
>    `MIN_DELTA_BEFORE_DEACTIVATION` (**7 días**) desde el primer fallo (`odoo/addons/base/models/ir_cron.py:24-25`, `MIN_FAILURE_COUNT_BEFORE_DEACTIVATION` v18,
>    `:36-37` v19; la lógica en `:474-488` v18). Además **avisa al admin**: *"Cron job … has been
>    deactivated after failing 5 times"*. Y trae dos campos nuevos para ello: `failure_count` y
>    `first_failure_date` (`:83-84`), que se resetean al primer éxito.
> 2. **Auto-desactivación voluntaria — el sustituto real de `numbercall=1`.** La API de progreso
>    acepta `deactivate`: `self.env['ir.cron']._notify_progress(done=…, remaining=…,
>    deactivate=True)` (`:734-749`), y el runner lo aplica al terminar —
>    `if status == CompletionStatus.FULLY_DONE and progress.deactivate: job['active'] = False`
>    (`:444-445`).
>
> **Para un cron de transportista esto importa**: un cron de actualización de tracking que falle
> contra la API del proveedor **se apaga solo a la semana**, y nadie lo relaciona con el proveedor
> caído — se ve como *"el cron dejó de ir"*. Si necesitas «una sola pasada», no busques `numbercall`:
> usa `_notify_progress(..., deactivate=True)`.


### Odoo 19.0 — delivery / stock_delivery (quick list)

The deltas vs 18.0 that break a carrier copied from an older version. **Full
detail + migration procedure: [migration-v18-to-v19.md](migration-v18-to-v19.md) §4.**

19. **`stock.quant.package` → `stock.package`** (core stock model renamed). Re-point
    every reference; packages of a picking: `picking.move_line_ids.result_package_id`.
20. **`_pre_put_in_pack_hook` moved** from `stock.picking` to `stock.package`, new
    signature; `_set_delivery_package_type` removed from the picking.
21. **`_apply_margins(self, price)` → `_apply_margins(self, price, order=False)`.**
22. **`available_carriers`/`_match*` take `source`, not `order`.**
23. **New `stock.picking._compute_allowed_carrier_ids`** (restricts selectable carriers).
24. **`send_shipping`** still returns `list[dict]` with `exact_price`/`tracking_number`.
25. **Delivery now ships payment integration** (`payment_provider`/`payment_transaction`/`ir_http`).
26. **Views/ORM:** `<list>` not `<tree>`; no `<group>` in `<search>`; `_compute_display_name`;
    `aggregator` not `group_operator`.
## Verified the hard way (2026-07)

27. **Green MOCKED tests do NOT prove the wire format — run a live read-only smoke test.**
    A carrier module with 116 green (mocked) tests was completely non-functional against the
    real API: it escaped the SOAP `docIn`, which the server rejected with HTTP 500. Mocks
    hid it. Before declaring "done", when the endpoint/test-env is reachable, hit it with a
    NON-mutating op (server date/time, tracking a fake ref, or an intentionally-bad-credentials
    call) and confirm the envelope/`docIn`/response-parsing actually round-trips. Add this to
    Success Criteria.
    **And the step before that: a mocked suite that has NEVER BEEN RUN proves not even that.**
    Its tests may be asserting false things — fixtures in a state the code under test discards,
    mocked responses knocked out of sync by a token cache — and all of them "pass". The order is:
    run it first, distrust it second. See workflow-steps.md STEP 5.
28. **SOAP `docIn`: RAW vs escaped, and result as child-nodes vs escaped-text.** On the
    WSintegracion/asmred/geslotrans platform the `docIn` must be embedded as **raw XML child
    nodes** (escaping the whole document ⇒ HTTP 500); individual field *values* are still
    escaped. The result may come back as **real child nodes** OR as escaped XML text — parse
    both (`if len(result_elem): element_to_dict(result_elem) else: parse(text)`). Errors arrive
    as `<RespuestaError><CodigoError><MensajeError>`. See `carrier-platform-map.md`.
29. **`xml.sax.saxutils.escape()` does NOT escape double quotes.** Values placed inside a
    double-quoted XML attribute (`referencia="{x}"`, `uidcliente="{x}"`) break the document if
    they contain `"`. Use `escape(x, {'"': "&quot;"})` (or `quoteattr`) for attribute values;
    plain `escape()` only for element text.
30. **Token-based carriers MUST retry once on HTTP 401 / session-expired code.** Even with a
    cached token (clock skew, server-side revocation) a call can 401. Pattern: on 401 (or the
    carrier's "session expired" code, e.g. `codigo_error=="1"`) invalidate the cached token and
    retry ONCE (`retry_auth=False` on the recursion) — never loop. (SEUR shipped without this;
    enlaza/maresa had it.)
31. **Use the base delivery helpers for weight/packages, not raw fields.** Read
    `picking._get_delivery_weight()` / `_get_delivery_packages_count()` (indaws_base_delivery),
    never `picking.shipping_weight` / `delivery_number_of_packages` directly — the raw form
    ignores the "manual weight/packages" overrides the base layer provides.
32. **Inbound webhooks: auto-generate the shared secret.** A `auth="public"` callback
    (`/carrier/webhook/<token>`) authenticated only by a token must NOT rely on a hand-typed
    secret (spoofing risk). Generate it with `secrets.token_urlsafe(32)` on create / when
    delivery_type is set, make the field readonly + a "Regenerate" button, and expose a
    computed full webhook URL to copy. Guard the empty-token case in the controller.
33. **Silent reference truncation collides.** Trimming `picking.name` to the carrier's max
    length silently can produce duplicate references. Warn (`_logger.warning`) — or raise if
    the reference is the carrier's UNIQUE key (some platforms dedupe by it). Note in code
    whether the reference is the identity or merely informational.
34. **A carrier's cancel/manifest op may not exist even if the sibling has it.** Verify each
    operation against the real WSDL/`?op=` page; TRANSLD had no `Anula` (use `BorrarEnvio`)
    nor `GetManifiesto` (build the manifest from Odoo pickings), both inherited-but-wrong from
    the asmred sibling.
35. **HttpCase/tour tests need a ThreadedServer (`workers=0`).** The `indaws/odoo` docker image
    in `run` mode starts a PreforkServer, so `HttpCase.setUpClass` fails with
    `'PreforkServer' object has no attribute 'httpd'`. Exclude those tests by tag when running
    in that harness (`--test-tags "/mod,-/mod:TestWebhook"`) or run them in a ThreadedServer.

## Verified the hard way (2026-08, building CBL)

36. **A carrier may have NO status API at all — seed the states you cause, don't map a code
    space that doesn't exist.** No query endpoint, no webhook, no public portal is a real and
    common configuration (CBL). Ask about all three in STEP 0. The module still needs
    `delivery.carrier.state` rows: create the ones your own code writes (registered / confirmed /
    cancelled). Without rows, `_set_delivery_state` finds no target, returns `False`, and every
    state write is dropped silently — how `indaws_delivery_ontime` broke.
    (`workflow-steps.md` STEP 3; `indaws_delivery_cbl/models/cbl_master_data.py`.)
37. **Daily token = calendar expiry, and the calendar is the CARRIER's.** A token valid "for
    today" has no `expires_in` to cache against: key the cache by **date**, and compute that date
    in the provider's timezone. On UTC, between 22:00 UTC and midnight you still think it is
    yesterday while the carrier has rolled over — a dead token reused for two hours, every day.
    (`provider-patterns.md` § "Daily Token"; `cbl_request.py::_get_daily_token` + `_cbl_today`.)
38. **No sandbox means `prod_environment` switches NOTHING — say so, loudly.** When the carrier
    publishes a single host, every call creates a real shipment a real van comes to collect. The
    standard test/production toggle then gives false safety: the user believes they are testing.
    This is a safety issue, not a style one. Either refuse to register while the toggle is off
    (`UserError` explaining there is no test environment) or make the absence explicit in the
    field help. (`delivery_carrier.py::_cbl_check_environment`.)
39. **Warnings are not errors, and dropping them is how `200 OK` comes to mean nothing.** Error
    handling that is binary (raise / continue) throws away the `warningList` that comes back with
    a successful response — and those warnings say the carrier *accepted the shipment but changed
    or ignored something* (weight over maximum, magnitude out of range, packages dropped). Never
    raise on them; never swallow them either: render them into the picking chatter.
    (`cbl_request.py::warnings_to_text` + `delivery_carrier.py::_cbl_post_warnings`.)
40. **Length dimensions default to MILLIMETRES in Odoo — convert them like weight.** Worse than
    the weight case because it fails silently: `product/models/product_template.py::
    _get_length_uom_id_from_ir_config_parameter` (v18 `:359 @99f5a33c64`, v19 `:367 @d1d161033`)
    returns mm. A carrier expecting metres gets a value 1000× too big with no exception — just
    wrong data, or a provider warning if you are lucky. (`workflow-steps.md` § "Length Conversion
    Pattern"; `delivery_carrier.py::_cbl_to_meters`.)
41. **"Cancel" may only FLAG the shipment — that is a third case, and it must reach the chatter.**
    Not "there is a cancel API" vs "there isn't": the API can accept the call and leave the actual
    cancellation to a person at the carrier, often depending on shipment state (pending = really
    deleted, confirmed = merely marked). If the operator does not read on the picking that a phone
    call is still pending, the shipment goes out. (`delivery_carrier.py::cbl_cancel_shipment`.)
42. **ZPL labels: the consequences, not just the encoding.** Storing ZPL as text (not base64 PDF)
    is only half of it. Odoo **cannot preview** a `.zpl` attachment — the user clicks and gets a
    download, so say in the chatter message that the labels are ZPL and go straight to a Zebra
    printer. And ZPL carriers typically return **one file per package**, not one per shipment:
    attach them individually, numbered (`<prefix>-<ref>-<n>.zpl`), and post a message when the
    carrier registers a shipment but returns no labels at all.
    (`delivery_carrier.py::_cbl_attach_labels`.)

## Verified the hard way (2026-08) — reviewing two carrier PRs against the providers' own docs

43. **Mock at the TRANSPORT boundary, not at the parser boundary.** Pitfall 27 says green mocked
    tests do not prove the wire; this is the cheap half of the cure, and it needs no live endpoint.
    An Ontime module had **38 green tests, every one of them patching `_send_request`** to return a
    dict — so the envelope handling was simply *not covered*. The provider actually answers a JSON
    **array**, the code did `response.get(key)` on it, and any real call died with
    `AttributeError: 'list' object has no attribute 'get'`. Patch `session.post` / `requests.post`
    with a fake response object instead (`status_code`, `text`, `json()` — ten lines) and the
    envelope enters coverage. Rule of thumb: **if your mock returns what the parser wants, you are
    testing the parser against itself.**
44. **The success criterion comes from the provider's SUCCESS SAMPLE, not from its error table.**
    Reading only the list of error codes invites "everything negative is an error", which is an
    *inference*. TRANSLD's sheet opens with *"0 (zero): Shipment inserted Ok. Any other value:
    Error on insertion"* and its own example of a good insertion answers `<Resultado return="0"/>`
    — while the module (and the v14 it was ported from, years in production) accepted anything
    non-negative, so two documented errors arrived as successful shipments with no tracking number.
    Ask for the sample of a CORRECT response, not just the error sheet. And when the legacy module
    disagrees with the doc and you cannot test a real registration, do not flip the criterion blind:
    keep the old one and **log a warning** on every value the doc does not call success, so the
    first real occurrence leaves a trace instead of a silent bad shipment.
45. **An assertion about what a value is NOT can stay green forever.** `assertNotIn("pre", url)`
    "protected" a production host that had been NXDOMAIN for months. Pin the literal
    (`assertTrue(url.startswith("https://<host>/"))`). Same shape as pitfall 27, one level down:
    a test can run, pass, and assert nothing.
46. **The production host is often NOT in the documentation** — only the pre/test one. All 14
    official Ontime documents give `gtsontimepre...` and never name a prod host, so the prod URL in
    the module came from nobody-remembers-where and eventually died. Before trusting one: `getent
    hosts <host>` plus a read-only HTTPS hit (an NXDOMAIN is a two-second check that saves a
    support ticket), pin it with a test (#45), and ask the provider to confirm it **in writing**.
47. **Master-data dictionaries lose entries when the module is ported to a new version.** Porting
    TRANSLD to 19.0 dropped 11 of 46 shipment error codes and 5 of 34 pickup ones. Nothing fails:
    the codes simply surface as `Unknown error (code -603)` the day a customer hits one. The check
    is mechanical and belongs in the migration checklist — count the keys of every dict in
    `<provider>_master_data.py` on both versions and diff the key sets, never eyeball them.
    (`migration-v18-to-v19.md` § Migration procedure, step 6.)

