# Discuss/Bus v19 Changes

## CRITICAL (Breaking Changes)

### BUS-C01: `bus.presence` model REMOVED
- **v18**: Full model in `bus/models/bus_presence.py` with fields `user_id`, `last_poll`, `last_presence`, `status`
- **v19**: File completely removed from `bus/models/__init__.py`; no longer exists in the bus module
- **Impact**: **BREAKING**. Any custom code that:
  - Reads/writes `bus.presence` records directly
  - Overrides `_update_presence`, `_send_presence`, `_gc_bus_presence`
  - Uses constants `UPDATE_PRESENCE_DELAY`, `DISCONNECTION_TIMER`, `AWAY_TIMER`, `PRESENCE_OUTDATED_TIMER`
  - References presence search domains like `[("user_id", "=", uid)]`

  The presence system has been completely redesigned. The `im_status` field, `_compute_im_status`, and `_is_user_available` have been removed from `bus/models/res_users.py`. Similarly, `im_status` and `_compute_im_status` have been removed from `bus/models/res_partner.py`. Presence is now likely managed entirely in the mail/discuss module.
- **Files**: `bus/models/bus_presence.py` (deleted), `bus/models/__init__.py`, `bus/models/res_users.py`, `bus/models/res_partner.py`

### BUS-C02: `res.users` stripped to minimal in bus module
- **v18**: Has `im_status` field, `_compute_im_status`, `_bus_channel` (returns `partner_id._bus_channel()`), `_is_user_available`
- **v19**: Only has `_bus_channel` (returns `self.partner_id` directly, no `._bus_channel()` chain)
- **Impact**: **BREAKING**. `im_status` is no longer provided by the bus module on `res.users`. The `_is_user_available()` method is removed. The `_bus_channel()` return value changed from `self.partner_id._bus_channel()` to `self.partner_id` directly. Custom code relying on `user.im_status` from the bus module or `_is_user_available()` must find the new provider (likely mail module).
- **Files**: `bus/models/res_users.py`

### BUS-C03: `res.partner` stripped to minimal in bus module
- **v18**: Has `im_status` field with `_compute_im_status` (computes from `bus.presence`)
- **v19**: Empty model, only inherits `bus.listener.mixin`. No `im_status` field.
- **Impact**: **BREAKING**. `partner.im_status` is no longer provided by the bus module. Custom code computing or reading `im_status` from bus must update.
- **Files**: `bus/models/res_partner.py`

### BUS-C04: `ir.websocket` presence system completely removed
- **v18**: Has methods `_get_missed_presences_identity_domains`, `_get_missed_presences_bus_target`, `_build_presence_channel_list`, `_update_bus_presence`, and `_prepare_subscribe_data` handles presence channels
- **v19**: All presence-related methods removed. `_prepare_subscribe_data` simplified to just build channels and return last id. New methods: `_serve_ir_websocket` (extensible event processing) and `_after_subscribe_data` (post-subscribe hook).
- **Impact**: **BREAKING**. Any module overriding presence-related methods on `ir.websocket` must completely rework. The presence subscription pattern (`odoo-presence-*` channels) is no longer handled in the bus module.
- **Files**: `bus/models/ir_websocket.py`

### BUS-C05: `_on_websocket_closed` changed to no-op hook
- **v18**: Sets user presence to "offline" when websocket closes
- **v19**: Empty method stub: "Modules can override this method to add custom behavior"
- **Impact**: Bus module no longer manages presence on websocket close. The behavior is expected to be handled by the mail module instead.
- **Files**: `bus/models/ir_websocket.py`

### BUS-C06: `_build_bus_channel_list` uses `all_group_ids` instead of `groups_id`
- **v18**: `channels.extend(self.env.user.groups_id)`
- **v19**: `channels.extend(self.env.user.all_group_ids)`
- **Impact**: Now includes implied/inherited groups in bus channel subscriptions. Custom code filtering bus notifications by group must account for this.
- **Files**: `bus/models/ir_websocket.py`

### BUS-C07: `bus.bus` model class renamed from `ImBus` to `BusBus`
- **v18**: `class ImBus(models.Model): _name = 'bus.bus'`
- **v19**: `class BusBus(models.Model): _name = 'bus.bus'`
- **Impact**: Custom code importing the class by name (rare but possible) must update. The model name `bus.bus` is unchanged.
- **Files**: `bus/models/bus.py`

### FSM-C08: `GC_UNLINK_LIMIT` imported in v19 bus.py
- **v18**: Uses `SQL` from `odoo.tools`
- **v19**: Additionally imports `GC_UNLINK_LIMIT` from `odoo.tools.constants` (though the GC still uses direct SQL DELETE)
- **Impact**: Minor. The GC comment was expanded to explain the direct SQL usage.
- **Files**: `bus/models/bus.py`

### BUS-C09: Websocket notification mechanism changed from socketpair to PriorityQueue
- **v18**: Uses `socket.socketpair()` (`__notif_sock_w`, `__notif_sock_r`) for notification signaling between threads
- **v19**: Uses `PollablePriorityQueue` (`__cmd_queue`) - a custom `PriorityQueue` with a selectable socket interface and command processing
- **Impact**: **BREAKING** for any code that interacts with websocket internals. New command pattern: `close()` sends commands via queue, `_disconnect` renamed from `disconnect`. The `_waiting_for_dispatch` flag is new. Imports added: `from itertools import count`, `from queue import PriorityQueue`.
- **Files**: `bus/websocket.py`

## Discuss Channel Changes

### DISC-C01: discuss.channel class renamed from `Channel` to `DiscussChannel`
- **v18**: `class Channel(models.Model): _name = 'discuss.channel'`
- **v19**: `class DiscussChannel(models.Model): _name = 'discuss.channel'`
- **Impact**: Custom code importing the class by name must update.
- **Files**: `mail/models/discuss/discuss_channel.py`

### DISC-C02: `_sql_constraints` migrated to `models.Constraint` / `models.Index`
- **v18**: `_sql_constraints = [("from_message_id_unique", ...), ("uuid_unique", ...), ...]`
- **v19**: `_from_message_id_unique = models.Constraint(...)`, `_uuid_unique = models.Constraint(...)`, `_group_public_id_check = models.Constraint(...)`
- **Impact**: Modern constraint pattern. Custom modules adding constraints to `discuss.channel` should use the new pattern.
- **Files**: `mail/models/discuss/discuss_channel.py`

### DISC-C03: `parent_channel_id` field: `auto_join` replaced with `bypass_search_access`
- **v18**: `parent_channel_id = fields.Many2one(..., auto_join=True, ...)`
- **v19**: `parent_channel_id = fields.Many2one(..., bypass_search_access=True, ...)`
- **Impact**: The `auto_join` parameter is likely deprecated/removed in v19 in favor of `bypass_search_access`.
- **Files**: `mail/models/discuss/discuss_channel.py`

### DISC-C04: `allow_public_upload` field REMOVED
- **v18**: `allow_public_upload = fields.Boolean(default=False)`
- **v19**: Field no longer present
- **Impact**: Custom code referencing this field will fail.
- **Files**: `mail/models/discuss/discuss_channel.py`

### DISC-C05: Discuss `ir_websocket.py` drastically simplified
- **v18**: Full override with `add_guest_to_context` decorator, guest handling, presence channels for guests, `_update_bus_presence` for guests, `_on_websocket_closed` for guests
- **v19**: Minimal override - only `_build_bus_channel_list` with guest/channel handling. No `add_guest_to_context` decorator. New "internal_users" subchannel instead of "members" subchannel.
- **Impact**: **BREAKING**.
  - `(c, "members")` subchannel renamed to `(c, "internal_users")`
  - `add_guest_to_context` decorator no longer used
  - Guest presence handling removed from discuss `ir_websocket.py`
  - `_update_bus_presence` override removed
  - `_on_websocket_closed` override removed
- **Files**: `mail/models/discuss/ir_websocket.py`

### DISC-C06: `group_public_id` becomes recursive computed field
- **v18**: `group_public_id = fields.Many2one('res.groups', ..., compute='_compute_group_public_id', readonly=False, store=True)`
- **v19**: `group_public_id = fields.Many2one('res.groups', ..., compute='_compute_group_public_id', recursive=True, readonly=False, store=True)`
- **Impact**: The `recursive=True` flag indicates the compute now depends on parent/child relationships.
- **Files**: `mail/models/discuss/discuss_channel.py`

## Notable (New Features / Improvements)

### DISC-N01: New `discuss.call.history` model
- **v19 only**: New model tracking call history per channel with fields: `channel_id`, `duration_hour` (computed), `start_dt`, `end_dt`, `start_call_message_id`
- Uses `models.Constraint` for NOT NULL checks and unique message constraint, `models.Index` for efficient queries
- New field on discuss.channel: `call_history_ids = fields.One2many("discuss.call.history", "channel_id")`
- **Impact**: Enables call history tracking. Custom modules extending discuss calls should use this model.
- **Files**: `mail/models/discuss/discuss_call_history.py`

### DISC-N02: New `ir_binary.py` in discuss models
- **v19 only**: New file added to discuss models
- **Impact**: Likely extends binary field handling for discuss attachments.
- **Files**: `mail/models/discuss/ir_binary.py`

### DISC-N03: New fields on `discuss.channel`
- `self_member_id`: computed Many2one to own channel member record (with `compute_sudo=True`)
- `invited_member_ids`: computed One2many for invited members (with `compute_sudo=True`)
- `message_count`: Integer computed field for message counting
- `channel_name_member_ids`: computed One2many for channel name computation
- `last_interest_dt`: now has a default value (`fields.Datetime.now() - timedelta(seconds=1)`)
- **Impact**: Enriched channel metadata. Custom code can leverage these new fields.
- **Files**: `mail/models/discuss/discuss_channel.py`

### DISC-N04: `is_member` field gains `compute_sudo=True`
- **v18**: `is_member = fields.Boolean('Is Member', compute='_compute_is_member', search='_search_is_member')`
- **v19**: `is_member = fields.Boolean("Is Member", compute="_compute_is_member", search="_search_is_member", compute_sudo=True)`
- **Impact**: Member check now runs with elevated privileges for performance.
- **Files**: `mail/models/discuss/discuss_channel.py`

### DISC-N05: `res.users` in discuss: `groups_id` replaced by `all_group_ids`
- **v18**: `search([("group_ids", "in", users.groups_id.ids)])`
- **v19**: `search([("group_ids", "in", users.all_group_ids.ids)])`; also `all_implied_ids` used for write
- **Impact**: Channel auto-subscription now correctly considers implied groups. Custom group-based channel logic should use `all_group_ids`.
- **Files**: `mail/models/discuss/res_users.py`

### DISC-N06: Store API changed from dict to method calls
- **v18**: `store.add({"initChannelsUnreadCounter": len(members_with_unread)})`, `store.add({...})`
- **v19**: `store.add_global_values(initChannelsUnreadCounter=len(members_with_unread))`, `store.add_global_values(...)`
- **Impact**: The `Store.add()` dict pattern is replaced by `Store.add_global_values()` with keyword args. Custom code extending `_init_messaging` or `_init_store_data` must update.
- **Files**: `mail/models/discuss/res_users.py`

### DISC-N07: New `is_in_call` field on `res.users`
- **v19 only**: `is_in_call = fields.Boolean("Is in call", related="partner_id.is_in_call")`
- **Impact**: Users now expose call status directly. Useful for presence/availability features.
- **Files**: `mail/models/discuss/res_users.py`

### DISC-N08: `hasCannedResponses` added to store data
- **v19 only**: Store now includes `hasCannedResponses` boolean computed from `mail.canned.response` search
- **Impact**: Client-side canned response feature detection. Group-based access check uses `all_group_ids`.
- **Files**: `mail/models/discuss/res_users.py`

### BUS-N01: `bus_listener_mixin._bus_send` now iterates with channel resolution loop
- **v18**: `main_channel = record._bus_channel(); main_channel.ensure_one()`
- **v19**: Adds a while loop resolving the channel chain: `while (new_main_channel := main_channel._bus_channel()) != main_channel: main_channel = new_main_channel`
- **Impact**: Supports multi-level channel delegation. Also `_bus_channel()` no longer calls `ensure_one()` on self (moved to `_bus_send`). Handles empty recordsets gracefully with `if not main_channel: continue`.
- **Files**: `bus/models/bus_listener_mixin.py`

### BUS-N02: Legacy test assets removed from bus manifest
- **v18**: Includes `web.tests_assets`, `web.qunit_suite_tests`, and legacy test removal patterns
- **v19**: Only `web.assets_unit_tests` with no legacy patterns
- **Impact**: Legacy JS test infrastructure fully removed.
- **Files**: `bus/__manifest__.py`

### BUS-N03: `debug` assets added to bus backend
- **v19**: `'bus/static/src/debug/**/*'` added to `web.assets_backend`
- **Impact**: New debug tooling for bus inspection.
- **Files**: `bus/__manifest__.py`

### BUS-N04: Worker assets pattern changed
- **v18**: Specific worker files listed: `websocket_worker.js`, `websocket_worker_utils.js`
- **v19**: Uses glob `'bus/static/src/workers/*'` with exclusion of `bus_worker_script.js`
- **Impact**: New worker script structure. The `bus_worker_script.js` is excluded from normal bundles.
- **Files**: `bus/__manifest__.py`

### BUS-N05: `_serve_ir_websocket` extensibility hook
- **v19 only**: New method on `ir.websocket` for processing websocket events by name
- **Impact**: Modules can now override this to handle custom websocket events without modifying core dispatch logic.
- **Files**: `bus/models/ir_websocket.py`

### BUS-N06: `lazy_property` import removed from websocket.py
- **v18**: `from odoo.tools import config, lazy_property`
- **v19**: `from odoo.tools import config` (lazy_property removed)
- **Impact**: Minor cleanup.
- **Files**: `bus/websocket.py`

### BUS-N07: SVG avatars changed from `<circle>` to `<rect>` in discuss channel
- **v18**: Channel/group avatars use `<circle>` SVG element
- **v19**: Changed to `<rect>` SVG element
- **Impact**: Visual change in default channel avatars. No functional impact.
- **Files**: `mail/models/discuss/discuss_channel.py`

### BUS-N08: New helper functions `is_channel` and `is_channel_or_group` in discuss_channel.py
- **v19 only**: Module-level predicate functions for filtering channels by type
- **Impact**: Utility functions available for import by other modules.
- **Files**: `mail/models/discuss/discuss_channel.py`

### BUS-N09: New imports in discuss_channel.py
- **v19**: Adds `MailDeliveryException`, `PUSH_NOTIFICATION_TYPE`, `Domain`, `email_normalize`, `hash_sign`, `OrderedSet`, `SQL`
- **Impact**: Indicates significant feature additions in the channel model (push notifications, improved email handling, domain class usage).
- **Files**: `mail/models/discuss/discuss_channel.py`
