# Scheduling: from "every Monday" to a repeat pattern

A schedule needs a **cron pattern** (five fields), a **time zone** and the task to run. The person
never says cron: you translate, say the result back in words and wait for a yes.

```
┌ minute (0-59)
│ ┌ hour (0-23)
│ │ ┌ day of the month (1-31)
│ │ │ ┌ month (1-12)
│ │ │ │ ┌ day of the week (0-7, 0 and 7 are Sunday; 1 is Monday)
* * * * *
```

| The person says | Pattern | Notes |
|---|---|---|
| every day at 8 | `0 8 * * *` | |
| weekdays at 8:30 | `30 8 * * 1-5` | |
| every Monday morning | `0 8 * * 1` | "morning" with no time: pick 08:00 and say so |
| every Monday and Thursday at 9 | `0 9 * * 1,4` | |
| every hour | `0 * * * *` | |
| every 2 hours | `0 */2 * * *` | |
| first of every month at 7 | `0 7 1 * *` | |
| the 1st and 15th | `0 7 1,15 * *` | |
| every quarter, first day | `0 7 1 1,4,7,10 *` | |
| once, on a given date | `0 9 12 11 *` with `max_runs: 1` | cron has no year; `max_runs: 1` makes it stop after that one |
| every 15 minutes | `*/15 * * * *` | see the cost note below |

What a pattern cannot say, and what to offer instead:

- **"Last day of the month"**: not expressible. Offer the 28th (`0 7 28 * *`) or ask whether the
  1st of the next month is fine.
- **"Every two weeks"**: not expressible. Offer the 1st and 15th, or weekly.
- **"Every weekday that is not a holiday"**: not expressible. Offer weekdays and say holidays run too.
- **Day of month and weekday together** (`0 8 1 * 1`): cron runs when *either* matches, not both.
  Never combine them unless the person truly means "or".
- **Below one minute**: impossible, and below an hour is rarely what anyone wants (below).

## Choosing the other fields

- **Time zone**: `Europe/Madrid` unless the person is somewhere else or says another city. State it
  in the confirmation. The wall-clock time holds across daylight saving.
- **Overlap**: `skip` (default) means a run is skipped if the previous one is still going. That is
  what almost everybody wants; use `allow` only if asked ("aunque la anterior no haya acabado").
- **End**: nothing unless they gave one. "Durante tres semanas" → `ends_at` three weeks from now;
  "tres veces" → `max_runs: 3`; "hasta el 30 de noviembre" → `ends_at`. Do not invent an end.
- **Start**: nothing unless they said "a partir del …" → `starts_at`.

## Cost and sanity

Every firing is a real run with a real cost, unattended, until someone pauses it. So:

- Anything more frequent than hourly: say what it implies (for example "that is 96 runs a day") and
  ask whether they are sure, once. Over daily: offer a stop date (the farm has no spending cap, so
  a stop date is the only brake).
- A team can hold 20 live schedules. If the answer is a `429` about schedules, list theirs and offer
  to delete one that is finished or no longer needed.
- Editing is not possible. To change anything: create the new one, confirm it, then delete the old
  one (never delete first).

## Confirmation to read back before creating

> Voy a programar **{what}** **{in words: every Monday at 08:00, Madrid time}**, empezando
> **{first run}**, sin fecha de fin. Si la anterior no ha terminado, se salta. ¿Lo creo?

After creating, take the first run from `next_fire_at` in the answer (it is UTC; convert to the
schedule's zone) and say it, so the person can see the pattern did what they meant.

## Pause, resume, delete

- **Pause**: reversible, the schedule stays. Resuming skips anything that fell in the paused window.
  This is the default reading of "párala", "que deje de hacerlo por ahora", "quítala de momento".
- **Delete**: permanent. Use only for "bórrala", "elimínala", "ya no la necesito", and still confirm.
- If they say "para eso" and both a running task and a schedule match, ask which one they mean.
- A `finished` schedule cannot be resumed: offer a new one.

## Run it now

`schedule fire <id>` starts one run immediately, outside the pattern ("lánzala ahora", "ejecuta la
programación X ya", "pruébala"). It is a real run with a real cost, so say so before doing it unless
they asked for exactly that. It does not move the next scheduled run and does not count as a missed
one; the task shows up in `schedule runs <id>` like any other. A paused schedule must be resumed
first; a finished one does not run any more (offer a new schedule); with `overlap: skip` it is refused
while the previous run is still going.

## Looking at what a schedule did

`schedule show <id>` shows the next run, how many runs happened, `missed_runs` (occurrences skipped
because the farm was down or the schedule was paused) and `last_fire_error` (a run that was skipped
and why). `schedule runs <id>` lists the tasks it started; each is a normal task, so `status`, `logs`
and `result` work on them.
