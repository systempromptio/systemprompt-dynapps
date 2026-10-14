<!-- verified: v18.0 @ 2026-06 -->
# Appointment — Core Reference (Odoo 18.0/19.0 **EE**)

> **EE-only module.** License OEEL-1 — does NOT exist in Community. Online booking on top of `calendar`.

Module path v18: `odoo_enterprise/18/appointment/`
Module path v19: `odoo_enterprise/19/appointment/`
License: OEEL-1. Depends on `calendar`, `phone_validation`, `portal`, `resource`, `web_gantt`.
Gotchas: `gotchas.md` core APG1-25 + APG83-100 (slots/capacity/cancel/invite/resource). Bridges APG26P-82MC in `integrations.md`.

## Models

| Model | File | Role |
|---|---|---|
| `appointment.type` | models/appointment_type.py (1484 LoC v18) | Booking template: slots, duration, assign rules, resources/users |
| `appointment.slot` | models/appointment_slot.py | Recurring weekday slot OR `unique` (custom) datetime range |
| `appointment.resource` | models/appointment_resource.py | Bookable thing (table/room) — extends `resource.resource` via mixin |
| `appointment.booking.line` | models/appointment_booking_line.py | M2M-with-extra-fields between `calendar.event` and `appointment.resource`, holds `capacity_reserved`/`capacity_used` |
| `appointment.invite` | models/appointment_invite.py | Shareable URL with `short_code` + filtered staff/resources |
| `appointment.question` | models/appointment_question.py | Pre-meeting form question (name/email/text/select/checkbox) |
| `appointment.answer.input` | (same model) | Visitor's answer per `calendar.event` |
| `appointment.answer` | models/appointment_answer.py | Predefined answer option for `select`/`checkbox` |
| `calendar.event` (inherit) | models/calendar_event.py (606 LoC) | Adds `appointment_type_id`, `appointment_status`, `access_token`, `booking_line_ids`, `resource_ids` |
| `calendar.attendee` (inherit) | models/calendar_attendee.py | Token-based portal cancel/confirm |
| `calendar.alarm` (inherit) | `default_for_new_appointment_type` flag |
| `res.partner` (inherit) | `calendar_verify_availability(date_start, date_end)` |

## Appointment categories (`appointment.type.category`)

| Category | Description | Slots used |
|---|---|---|
| `recurring` | Default. Weekly recurring slots | `appointment.slot` slot_type=recurring |
| `punctual` | Limited between `start_datetime`/`end_datetime` | recurring slots within bounds |
| `custom` | Hand-picked one-off ranges | `slot_type='unique'` slots with `start_datetime`/`end_datetime` |
| `anytime` | Shared full calendar (one per user) | recurring 7-day default |

`_compute_category` (`appointment/models/appointment_type.py:242-246`, `AppointmentType._compute_category`) ONLY decides `punctual` (if `start_datetime`/`end_datetime` set) vs `recurring`, and auto-creates default slots when none exist. `custom`/`anytime` are set via `_inverse_category` (`:248-254`), not the compute (see APG95). Only one `anytime` per user is enforced via `_check_staff_user_configuration` (`:349`).

## Booking lifecycle

`calendar.event.appointment_status`:

```
request → booked → attended
                 ↘ no_show
                 ↘ cancelled
```

- `booked` is the default (`_compute_appointment_status` `:126-131`: sets `booked` if a type is present and status empty). `request` only via `_get_default_appointment_status`-style logic (`:580-596`): when `appointment_manual_confirmation` is set, and for resource types only when `used/total_capacity > resource_manual_confirmation_percentage` (`float_compare(...) > 0`, default 0).
- Setting `active=False` writes `appointment_status='cancelled'` (and vice-versa) — create+write at `appointment/models/calendar_event.py:222-237`, `CalendarEvent.create`.
- `_compute_appointment_status` cascades type changes (type removed → status `False`); `attended`/`no_show` set by manager (gantt buttons).
- `_send_invitation_emails()` lives on `calendar.attendee` (`appointment/models/calendar_attendee.py:16`, `Attendee._send_invitation_emails`), invoked as `confirmed_bookings.attendee_ids._send_invitation_emails()` from `calendar.event.write` only on the `→booked` transition (APG7).

## Slot generation

`_slots_generate(first_day, last_day, timezone, reference_date)` (`appointment/models/appointment_type.py:598`, `AppointmentType._slots_generate`):
1. Project recurring slots in `appointment_tz` between bounds.
2. Skip slots `< reference_date + min_schedule_hours` (with rolling `local_start += duration`).
3. For `custom`: emit unique slots whose `end_datetime > reference_date`.
4. Output dict per slot with `(start, end)` in UTC, appointment_tz, visitor_tz.

`_get_appointment_slots(timezone, filter_users, filter_resources, asked_capacity, reference_date)` produces the calendar grid for the website. Calls `_slots_fill_users_availability` (line 1037) or `_slots_fill_resources_availability` (line 1213) to mark each slot.

## Assignment methods (v18)

`assign_method` selection:
- `resource_time` (default): visitor first picks user/resource, then time. URL contains `staff_user_id` / `resource_id`.
- `time_resource`: visitor first picks time, then sees who's available. Slot dict gets `available_staff_users` / `available_resource_ids`.
- `time_auto_assign`: random/round-robin pick at booking submit (`_get_random_user_id`).

v19 replaces this with `assignment_method` selection (`auto`/`manual`) + `is_auto_assign` + `is_date_first` + `select_first` (`date`/`user_resource`). See v19_changes.md (APG13).

## Resources & capacity

`appointment.resource` extends `resource.mixin`. Fields: `capacity` (≥1), `shareable` (multiple attendees same slot), `linked_resource_ids` (combinable resources, e.g., 2 small tables → 1 big).

Booking lines (`appointment.booking.line`) compute `capacity_used`:
- shareable + manage_capacity → `capacity_used = capacity_reserved`
- non-shareable OR no manage_capacity → `capacity_used = resource.capacity` (whole resource locked)

`_get_filtered_possible_capacity_combinations(asked_capacity)` finds resource combinations that match a requested capacity.

## Manual confirmation (v18)

`appointment_manual_confirmation = True` → bookings start as `request`. For resources, only triggers if `total_used / total_capacity > resource_manual_confirmation_percentage` (default 0). v19 inverts the logic (APG14): `auto_confirm = True` (default) + `manual_confirmation_percentage` (default 1.0).

## Invitations

`appointment.invite`:
- `short_code` (unique) → URL `/book/<short_code>`.
- `appointment_type_ids` + filter `staff_user_ids` / `resource_ids` (filtered subset of the type's full pool).
- `_compute_short_code_warning` flags duplicates and bad chars (`[A-Za-z0-9_-]+` only).
- `redirect_url` builds final shareable link.

## Email templates

- `attendee_invitation_mail_template` — sent to attendee on booking (default `booked_mail_template_id`).
- `appointment_canceled_mail_template` — sent on cancel (default `canceled_mail_template_id`).
- `_send_invitation_emails()` triggered when `appointment_status` flips to `booked`.
- Discuss videocall: `_compute_videocall_source` (`appointment/models/calendar_event.py:198-206`, `class CalendarEvent`) copies `appointment_type_id.event_videocall_source` onto the event; `_compute_videocall_redirection` (`:188-196`) then builds `/calendar/videocall/<access_token>` from `videocall_location`+`access_token` (see APG19).

## Access tokens

Every `calendar.event` carries `access_token` (uuid4) for portal flows. `_init_column` skips bulk-init (lazy-generated). Routes:
- `/calendar/view/<access_token>` — guest portal view, cancel/reschedule.
- `/calendar/videocall/<access_token>` — Discuss room redirect.

## Reminders

`appointment.type.reminder_ids` → `calendar.alarm`. Default reminders propagate to events on creation via `_compute_alarm_ids` (only fills if event has none).

## Country gating

`appointment.type.country_ids` empty = all countries. Check happens in controller using GeoIP on the visitor.

## Security groups

- `appointment.group_appointment_user` — see/manage own types.
- `appointment.group_appointment_manager` — manage all types and global config.

Record rules per `security/ir_rule_data.xml` restrict by `staff_user_ids` or `create_uid`.

## Calendar integrations (CRM/Studio/HR)

Standalone `appointment` only. Bridges in:
- `appointment_account_payment` — paid bookings.
- `appointment_crm` — auto-create lead on booking.
- `appointment_hr` — anytime category uses HR working hours.
- `appointment_sms` — SMS reminders.
- `website_appointment` — website visibility/`is_published`.
