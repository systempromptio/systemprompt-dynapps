# Local Kubernetes load test, 2026-10-09

This report covers the gateway on a local kind cluster running the real Helm chart, with mock inference and no paid calls. It compares the 0.64.0 release ("baseline") with the hot-path build ("patched": astound `next` fd52bbb0 on core `next` e06ff0e9c, image `systemprompt-astound:loadtest-local`). Rig: `deploy/loadtest/kind/README.md`.

## Result

The patched image raised the ceiling from 20 to 50 concurrent requests, from 29 to 79 rps with HPA off. p99 gateway overhead fell from 50 ms to 25 ms at every step both images passed. The errors that dominated the baseline's fixed-load scenarios are gone: every patched scenario ran at 0% errors, apart from the deliberate 429s in quota enforce.

The next ceiling is set by quota accounting. Every request upserts the same `ai_quota_buckets` rows, so at 100 or more concurrent the upsert averages 4 to 66 ms and sessions queue on row locks. The audit sealer job on the patched image also never seals; see blocker 2.

## How the numbers were taken

- **Cluster:** 1 control plane plus 3 workers, CPU-capped at 2 + 5 + 5 + 4. Postgres 16 sits alone on the 4-CPU worker with an 8 Gi limit. The gateway runs 3 replicas at 500m request and 2 CPU limit; the admin is a single replica.
- **Inference:** core `crates/tests/mock-inference`, 300 to 900 ms latency, 32 output tokens. Its own primary and EU deployments act as failover.
- **Load:** in-cluster `hey` Jobs, one process per seeded user's PAT and session. Each run is 2 minutes, preceded by a reset of the app's database connections and a 20 s warm-up outside the measured window.
- **Stop rule:** any non-200, or p99 `gateway_overhead_seconds` above 200 ms. A run that is not majority 200 is marked invalid and is never a ceiling.
- **Histogram limits:** overhead buckets are 25/50/100/250/500/1000 ms, so quantiles are bucket-interpolated and anything past 1 s reads as 1000.
- **Sanity reference:** attempt 1 measured 10 concurrent at 14.5 rps and 86 ms p99 on cold pools. Attempt 5 measured 14.8 rps and 50 ms p99 with the warm-up; the difference is cold reconnects.
- **Baseline only:** `max_lifetime_secs: 120` recycled connections. Without it, Postgres was OOM-killed after about 5 minutes; see blocker 1. Every 0.64.0 `/v1/messages` call also made one HTTP hop to the admin pod for authorization, through a socat relay, because 0.64.0 serves that check only on the admin role.
- **Patched only:** the in-process authz hook (`mode: extension`), statement cache 100, no pool recycle. Routes were not pre-restored. The admin was restarted on purpose between s1 and s2, and both routes stayed open (`reports/.../patched-0.65-attempt1-s1/admin-restart-routes.txt`).
- **Valid runs:** baseline `reports/loadtest/kind/baseline-0.64.0-attempt5`; patched `patched-0.65-attempt1-{s1,rest,s9-enforce-fixed}`. Attempts 2 to 4 are kept and labelled INVALID or HOST-CONTENDED.

## Blockers found, ranked

1. **Unbounded Postgres backend memory on 0.64.0. Fixed in core.** sqlx with statement cache 0 never closed named prepared statements. Postgres grew about 25 MB/s at 10 to 20 concurrent clients, each backend reached about 190 MB, and Postgres was OOM-killed within 5 minutes. With cache 100, patched Postgres stayed at 2.0 to 5.9 Gi under every scenario with zero restarts.
2. **The audit chain sealer job never seals on the patched image. Open.** `audit_chain_seal` runs every 10 s and always returns "another replica is sealing", although only one admin runs the scheduler. The same SQL run by hand seals rows. `audit_chain_unsealed('ai_requests')` grew to 232,088 by the end of the matrix; see the seal-lag list. The request path no longer waits on the chain, but tamper evidence for new rows is not being produced.
3. **Quota bucket hot rows. Open; this is now the ceiling.** In the patched s5-chain-on run at 400 concurrent, sessions averaged 42 waiting on `transactionid`, 63 on the LockManager lightweight lock, and 7 on `ai_quota_buckets` tuple locks. The upsert averaged 66 ms in the patched s5-chain-off run. The organization and project windows are single rows that every request updates. This is plan item 2b.7, "quota windows in one statement", or sharded counters.
4. **Admin authz hop on 0.64.0. Fixed by the in-process hook.** Under load the relay hop failed outright: 403 `authz_hook_fault` "transport failure" on 5 to 40% of requests in s4, s5, s6, s8 and s10. Admin CPU fell from 150–500m to 2–7m.
5. **Audit chain head lock on 0.64.0. Fixed by the sealer.** At 400 concurrent the baseline averaged 31 sessions on `audit_chain_heads` tuple locks and 51 on `transactionid`. The patched image has no chain-head waits.
6. **Route entitlement flipped closed on admin restart (0.64.0). Fixed.** The catalog sync rewrote gateway routes to `default_included = false` over rules.yaml's `default: open`, and core's rule check sees `roles []` for a PAT caller. Every PAT call returned 403 after an admin restart. Patched: unchanged across a deliberate restart.
7. **Background refresh jobs compete with traffic.** `refresh_conversation_facts` and the conversation rollup take 6 to 14 s per call on the admin and lead total DB time at low concurrency on both images.
8. **Chart gaps found on first boot. Fixed in the chart; they apply on GKE too.**
   - The `/app/storage` emptyDir hid the image's assets.
   - The authz hook needed loopback, which is now in-process.
   - Plain-http hosts need `pod.trustedHttpHosts`.
   - `render-profile.sh` omits `encryption_master_key`.
   - The policies file only seeds an empty `ai_gateway_policies` table, so changing it has no effect on a running install. The rig edits the row instead.

## Before and after

### Ceilings (last ramp step that passed the stop rule)

| ramp | 0.64.0 | patched | rps at ceiling (0.64.0 → patched) |
|---|---|---|---|
| s1-ramp-buffered-hpa-off | 20 | 50 | 29.1 → 79.4 |
| s1-ramp-buffered-hpa-on | 10 | 50 | 14.6 → 79.1 |
| s2-ramp-stream | 20 | 30 | 27.3 → 44.9 |

HPA behaviour: the baseline never scaled; it stayed at 3 replicas through its passing steps, because the stop rule tripped first at 20 concurrent. The patched image scaled from 3 to 5 gateways during its 100-concurrent step, at 49% CPU against the 65% target by the end. Scaling did not help, because the limit is Postgres row contention, not gateway CPU. The ceiling with HPA on equals the ceiling with HPA off: 50 concurrent, 79 rps. Patched streaming reached 30 concurrent at 44.9 rps, against 20 at 27.3 rps on the baseline.

Notes on the per-scenario table:
- **"DB stmts/req"** is every statement the app role ran in the window, divided by HTTP requests, so it includes the admin's background jobs. It is meaningful from about 20 rps up. At 30 concurrent it fell from 180 to 62.
- **Invalid runs:**
  - Baseline s5-chain-off and s7-saturation are invalid; s7 was 99.5% 503 load-shed by design.
  - Patched s9 is invalid by the majority-200 rule because it is a denial test: 1,800 OK and 16,422 429.
  - Patched s7 lost its generator logs; Prometheus counted 423,108 shed responses and 6,364 served.
- **Baseline s9 never enforced.** The policy file never reached the live row, as described in blocker 8, and 0.64.0 cannot be redeployed on the migrated database. Quota enforcement is therefore measured on the patched image only.
- **Patched s5-chain-off ran slower than chain-on.** That is quota-row contention varying between runs, not the chain, which is off the request path on this image.

### Per scenario

| run | image | rps | errors | e2e p50/p99 ms | overhead p50/p99 ms | PG CPU avg/max m | PG mem max Mi | gateway CPU avg m | admin CPU avg m | DB stmts/req | PG restarts |
|---|---|---|---|---|---|---|---|---|---|---|---|
| s1-ramp-buffered-hpa-off-c2 | 0.64.0 | 3.0 | 0.0% | 646/965 | 38/50 | 320/648 | 2211 | 35 | 32 | n/a | 0 |
| s1-ramp-buffered-hpa-off-c2 | patched | 3.2 | 0.0% | 622/914 | 16/25 | 209/568 | 2038 | 13 | 2 | 69.3 | 0 |
| s1-ramp-buffered-hpa-off-c5 | 0.64.0 | 7.3 | 0.0% | 681/966 | 38/50 | 493/768 | 2314 | 61 | 54 | n/a | 0 |
| s1-ramp-buffered-hpa-off-c5 | patched | 8.0 | 0.0% | 616/913 | 8/24 | 246/501 | 2156 | 22 | 2 | 62.9 | 0 |
| s1-ramp-buffered-hpa-off-c10 | 0.64.0 | 14.8 | 0.0% | 675/970 | 38/50 | 693/1064 | 2870 | 110 | 102 | n/a | 0 |
| s1-ramp-buffered-hpa-off-c10 | patched | 16.1 | 0.0% | 624/913 | 9/25 | 355/667 | 2369 | 42 | 3 | 63.4 | 0 |
| s1-ramp-buffered-hpa-off-c20 | 0.64.0 | 29.1 | 0.0% | 685/981 | 38/85 | 1336/1828 | 4462 | 209 | 182 | None | 0 |
| s1-ramp-buffered-hpa-off-c20 | patched | 32.1 | 0.0% | 619/914 | 11/25 | 421/786 | 2371 | 78 | 2 | 62.1 | 0 |
| s1-ramp-buffered-hpa-off-c30 | 0.64.0 | 42.5 | 0.0% | 703/1066 | 40/228 | 2094/2857 | 7005 | 319 | 285 | 180.4 | 0 |
| s1-ramp-buffered-hpa-off-c30 | patched | 48.1 | 0.0% | 617/919 | 15/40 | 595/807 | 2480 | 109 | 2 | 61.6 | 0 |
| s1-ramp-buffered-hpa-off-c50 | patched | 79.4 | 0.0% | 627/926 | 18/50 | 806/1344 | 2753 | 181 | 2 | 62.6 | 0 |
| s1-ramp-buffered-hpa-off-c100 | patched | 109.4 | 0.0% | 710/3119 | 69/1000 | 1272/2018 | 3506 | 314 | 2 | 62.8 | 0 |
| s1-ramp-buffered-hpa-on-c2 | 0.64.0 | 2.9 | 0.0% | 685/966 | 37/50 | 352/1134 | 2252 | 32 | 36 | 128.3 | 0 |
| s1-ramp-buffered-hpa-on-c2 | patched | 3.1 | 0.0% | 656/939 | 29/63 | 289/550 | 2880 | 13 | 2 | 69.7 | 0 |
| s1-ramp-buffered-hpa-on-c5 | 0.64.0 | 7.3 | 0.0% | 674/968 | 38/50 | 449/721 | 2457 | 54 | 46 | 126.4 | 0 |
| s1-ramp-buffered-hpa-on-c5 | patched | 8.1 | 0.0% | 612/912 | 17/25 | 258/528 | 2939 | 24 | 2 | 66.0 | 0 |
| s1-ramp-buffered-hpa-on-c10 | 0.64.0 | 14.6 | 0.0% | 678/975 | 38/123 | 697/1052 | 3212 | 110 | 93 | 124.8 | 0 |
| s1-ramp-buffered-hpa-on-c10 | patched | 16.1 | 0.0% | 610/916 | 16/25 | 315/670 | 2949 | 43 | 2 | 64.2 | 0 |
| s1-ramp-buffered-hpa-on-c20 | 0.64.0 | 24.4 | 0.0% | 814/1177 | 173/462 | 1174/1765 | 4396 | 192 | 162 | 124.9 | 0 |
| s1-ramp-buffered-hpa-on-c20 | patched | 31.9 | 0.0% | 630/917 | 17/25 | 452/912 | 2954 | 76 | 2 | 65.0 | 0 |
| s1-ramp-buffered-hpa-on-c30 | patched | 48.1 | 0.0% | 619/918 | 17/44 | 618/1056 | 3136 | 120 | 3 | 63.6 | 0 |
| s1-ramp-buffered-hpa-on-c50 | patched | 79.1 | 0.0% | 631/923 | 18/50 | 841/1282 | 3212 | 198 | 2 | 64.2 | 0 |
| s1-ramp-buffered-hpa-on-c100 | patched | 116.0 | 0.0% | 704/3053 | 68/1000 | 1557/2489 | 4128 | 268 | 3 | 64.0 | 0 |
| s10-pod-kill | 0.64.0 | 45.8 | 11.4% | 3704/11280 | 1000/1000 | 3285/3878 | 6477 | 471 | 373 | 121.7 | 0 |
| s10-pod-kill | patched | 105.1 | 0.0% | 1644/5320 | 816/1000 | 2218/3888 | 3867 | 377 | 3 | 63.9 | 0 |
| s2-ramp-stream-c2 | 0.64.0 | 2.7 | 0.0% | 772/1012 | 39/96 | 291/743 | 2369 | 35 | 22 | 130.7 | 0 |
| s2-ramp-stream-c2 | patched | 3.0 | 0.0% | 672/956 | 13/25 | 218/546 | 3628 | 14 | 2 | 71.1 | 0 |
| s2-ramp-stream-c5 | 0.64.0 | 7.0 | 0.0% | 712/1012 | 38/50 | 524/1041 | 2769 | 64 | 52 | 127.3 | 0 |
| s2-ramp-stream-c5 | patched | 7.6 | 0.0% | 660/952 | 15/25 | 329/675 | 3711 | 26 | 2 | 66.8 | 0 |
| s2-ramp-stream-c10 | 0.64.0 | 13.8 | 0.0% | 728/1012 | 38/50 | 785/1024 | 3370 | 107 | 94 | 126.0 | 0 |
| s2-ramp-stream-c10 | patched | 15.0 | 0.0% | 672/960 | 15/25 | 348/877 | 3744 | 46 | 2 | 64.9 | 0 |
| s2-ramp-stream-c20 | 0.64.0 | 27.3 | 0.0% | 728/1024 | 38/78 | 1331/1765 | 5035 | 211 | 182 | 126.5 | 0 |
| s2-ramp-stream-c20 | patched | 29.9 | 0.0% | 668/956 | 10/25 | 423/793 | 3821 | 77 | 5 | 63.8 | 0 |
| s2-ramp-stream-c30 | 0.64.0 | 36.3 | 0.0% | 796/1600 | 42/951 | 1753/2349 | 6537 | 290 | 236 | 125.8 | 0 |
| s2-ramp-stream-c30 | patched | 44.9 | 0.0% | 668/956 | 12/25 | 646/1152 | 4056 | 120 | 2 | 62.1 | 0 |
| s2-ramp-stream-c50 | patched | 40.1 | 0.0% | 1276/1876 | 620/1000 | 550/1249 | 4078 | 119 | 3 | 62.6 | 0 |
| s3-long-conversation | 0.64.0 | 26.9 | 12.1% | 5773/19651 | 1000/1000 | 2854/3875 | 7782 | 369 | 213 | 37.1 | 1 |
| s3-long-conversation | patched | 64.3 | 0.0% | 1788/9175 | 1000/1000 | 1142/2223 | 4517 | 255 | 2 | 104.3 | 0 |
| s4-hot-quota | 0.64.0 | 50.4 | 25.0% | 7786/17641 | 1000/1000 | 3322/3882 | 6670 | 418 | 367 | 114.0 | 0 |
| s4-hot-quota | patched | 125.7 | 0.0% | 2400/10475 | 1000/1000 | 1752/2856 | 4557 | 601 | 3 | 63.1 | 0 |
| s5-chain-off | 0.64.0 | 85.7 | 90.2% (invalid) | 6446/8919 | 1000/1000 | 3032/3872 | 6805 | 469 | 294 | 70.6 | 0 |
| s5-chain-off | patched | 39.4 | 0.0% | 10297/13686 | 1000/1000 | 794/1275 | 4947 | 148 | 2 | 69.5 | 0 |
| s5-chain-on | 0.64.0 | 49.2 | 22.3% | 7799/17440 | 1000/1000 | 3162/3880 | 6573 | 435 | 357 | 116.1 | 0 |
| s5-chain-on | patched | 84.3 | 0.0% | 4174/11440 | 1000/1000 | 3175/3868 | 4715 | 337 | 3 | 66.4 | 0 |
| s6-failover | 0.64.0 | 44.8 | 5.6% | 3673/14538 | 1000/1000 | 3023/3876 | 7439 | 395 | 336 | 125.1 | 0 |
| s6-failover | patched | 123.8 | 0.0% | 1425/5620 | 810/1000 | 1908/3142 | 5148 | 394 | 2 | 66.5 | 0 |
| s7-saturation | 0.64.0 | 3362.9 | 99.5% (invalid) | 10061/27944 | 1000/1000 | 2627/3867 | 7517 | 780 | 151 | 0.7 | 1 |
| s7-saturation | patched | None | n/a | n/a/n/a | 380/965 | 1219/3855 | 5416 | 351 | 2 | None | 0 |
| s8-pool-starvation | 0.64.0 | 83.9 | 43.7% | 2733/3902 | 908/1000 | 3130/3890 | 6619 | 643 | 495 | 105.4 | 0 |
| s8-pool-starvation | patched | 152.0 | 0.0% | 1309/1676 | 740/995 | 1482/2614 | 4509 | 396 | 2 | 64.3 | 0 |
| s9-quota-enforce | 0.64.0 | 33.1 | 5.1% | 1863/5666 | 865/1000 | 1744/3875 | 7117 | 275 | 263 | 16.4 | 1 |
| s9-quota-enforce | patched | 151.4 | 90.1% (invalid) | 732/1586 | 82/429 | 1402/1953 | 5897 | 348 | 7 | 50.9 | 0 |

### Top statements by total time, s1-ramp-buffered-hpa-off-c20, 0.64.0

| total ms | calls | mean ms | query |
|---|---|---|---|
| 12590.6 | 2 | 6295.307 | `SELECT refresh_conversation_facts_pending() AS "written!"` |
| 12579.7 | 2 | 6289.855 | `SELECT refresh_conversation_facts(ARRAY( SELECT DISTINCT canonical_conversation_context(context_id) ` |
| 10318.4 | 2 | 5159.219 | `WITH requests AS MATERIALIZED ( SELECT r.*, ar.wire_protocol, ar.cache_read_tokens, ar.cache_creatio` |
| 4317.5 | 3520 | 1.227 | `INSERT INTO ai_requests ( id, request_id, user_id, session_id, task_id, context_id, gateway_conversa` |
| 2310.7 | 3520 | 0.656 | `SELECT audit_chain_append('ai_requests', NEW.id, NEW.created_at, audit_chain_payload_ai_requests(NEW` |
| 2066.2 | 2 | 1033.088 | `DELETE FROM conversation_facts f WHERE f.context_id = ANY(context_ids) AND NOT EXISTS (SELECT $2 FRO` |
| 1584.6 | 3520 | 0.450 | `INSERT INTO ai_request_attributions (request_id, dimension, value, source) SELECT $1, t.dimension, t` |
| 1397.3 | 3520 | 0.397 | `INSERT INTO ai_request_messages (request_id, role, content, sequence_number) SELECT $1::varchar, $3,` |
| 1385.3 | 21120 | 0.066 | `SELECT u.id AS "id!", u.name AS "username!", u.email AS "email!", COALESCE(u.display_name, u.full_na` |
| 1330.3 | 7402 | 0.180 | `INSERT INTO governance_decisions (id, user_id, session_id, tool_name, agent_id, agent_scope, decisio` |

### Postgres waits, s1-ramp-buffered-hpa-off-c20, 0.64.0 (avg active sessions per 1 s sample)

| type | event | relation | avg |
|---|---|---|---|
| CPU | active |  | 0.71 |
| IO | WALSync |  | 0.58 |
| Client | ClientRead |  | 0.52 |
| LWLock | WALWrite |  | 0.07 |
| CPU | idle in transaction |  | 0.03 |
| IO | WALWrite |  | 0.03 |
| Timeout | VacuumDelay |  | 0.03 |
| Lock | transactionid |  | 0.01 |

### Top statements by total time, s1-ramp-buffered-hpa-off-c20, patched

| total ms | calls | mean ms | query |
|---|---|---|---|
| 15510.4 | 2 | 7755.209 | `SELECT refresh_conversation_facts_pending() AS "written!"` |
| 15497.6 | 2 | 7748.793 | `SELECT refresh_conversation_facts(ARRAY( SELECT DISTINCT canonical_conversation_context(context_id) ` |
| 12535.1 | 2 | 6267.561 | `WITH requests AS MATERIALIZED ( SELECT r.*, ar.wire_protocol, ar.cache_read_tokens, ar.cache_creatio` |
| 2727.8 | 2 | 1363.907 | `DELETE FROM conversation_facts f WHERE f.context_id = ANY(context_ids) AND NOT EXISTS (SELECT $2 FRO` |
| 2016.4 | 3872 | 0.521 | `INSERT INTO ai_requests ( id, request_id, user_id, session_id, task_id, context_id, gateway_conversa` |
| 1465.9 | 3872 | 0.379 | `INSERT INTO ai_request_attributions (request_id, dimension, value, source) SELECT $1, t.dimension, t` |
| 1349.5 | 7928 | 0.170 | `INSERT INTO governance_decisions (id, user_id, session_id, tool_name, agent_id, agent_scope, decisio` |
| 1298.3 | 3872 | 0.335 | `INSERT INTO ai_request_messages (request_id, role, content, sequence_number) SELECT $1::varchar, 'as` |
| 1199.1 | 3872 | 0.310 | `UPDATE ai_requests SET input_tokens = $2, output_tokens = $3, cache_read_tokens = $4, cache_creation` |
| 1046.9 | 3872 | 0.270 | `INSERT INTO ai_request_messages (id, request_id, role, content, sequence_number, created_at, updated` |

### Postgres waits, s1-ramp-buffered-hpa-off-c20, patched (avg active sessions per 1 s sample)

| type | event | relation | avg |
|---|---|---|---|
| CPU | active |  | 0.68 |
| IO | WALSync |  | 0.35 |
| Client | ClientRead |  | 0.35 |
| CPU | idle in transaction |  | 0.09 |
| LWLock | WALWrite |  | 0.03 |

### Postgres waits under fixed load (s5-chain-on, 400 concurrent; avg sessions)

| wait | 0.64.0 | patched |
|---|---|---|
| Lock transactionid | 50.83 | 42.37 |
| Lock tuple audit_chain_heads | 31.12 | 0 |
| LWLock LockManager | 28.19 | 63.02 |
| Lock tuple ai_quota_buckets | below top 6 | 6.96 |

### Seal lag (audit_chain_unsealed ai_requests, ai_safety_findings) after each patched run

- s1-ramp-buffered-hpa-off-c2: 782	0
- s1-ramp-buffered-hpa-off-c5: 2156	0
- s1-ramp-buffered-hpa-off-c10: 4498	1
- s1-ramp-buffered-hpa-off-c20: 8750	1
- s1-ramp-buffered-hpa-off-c30: 14949	2
- s1-ramp-buffered-hpa-off-c50: 24921	2
- s1-ramp-buffered-hpa-off-c100: 38745	3
- s1-ramp-buffered-hpa-on-c2: 39478	3
- s1-ramp-buffered-hpa-on-c5: 40836	3
- s1-ramp-buffered-hpa-on-c10: 43172	3
- s1-ramp-buffered-hpa-on-c20: 47418	3
- s1-ramp-buffered-hpa-on-c30: 53617	3
- s1-ramp-buffered-hpa-on-c50: 63564	3
- s1-ramp-buffered-hpa-on-c100: 78155	5
- s10-pod-kill: 91375	6
- s2-ramp-stream-c2: 92127	6
- s2-ramp-stream-c5: 93433	6
- s2-ramp-stream-c10: 95637	6
- s2-ramp-stream-c20: 99631	7
- s2-ramp-stream-c30: 105452	7
- s2-ramp-stream-c50: 110711	9
- s3-long-conversation: 119144	11
- s4-hot-quota: 135861	14
- s5-chain-off: 152239	17
- s5-chain-on: 146736	17
- s6-failover: 167744	23
- s7-saturation: 174551	23
- s8-pool-starvation: 193400	24
- s9-quota-enforce: 232088	26


### Zero external calls

The only providers on the rig are the two in-cluster mocks. Every `gateway_deployment_selected_total` series in every run names `mock-anthropic`. Across the valid runs, the mocks' `/stats` counters served 265,422 requests: 245,051 on primary and 20,371 on EU during failover. The generators recorded 259,058 HTTP 200s. The difference is the per-run warm-up traffic and the patched s7 run whose generator logs were lost.

## Re-run

```
just loadtest-kind-up && just loadtest-kind-deploy
just loadtest-kind-seed 100 && just loadtest-kind-seed 20 && just loadtest-kind-seed 100 hot commerce
LOADTEST_OUT=reports/loadtest/kind/<new-dir> just loadtest-kind-matrix
# patched image
LOADTEST_IMAGE_TAG=loadtest-local LOADTEST_VALUES=values-kind-patched.yaml LOADTEST_NO_ROUTE_RESTORE=1 \
  LOADTEST_OUT=reports/loadtest/kind/<new-dir> just loadtest-kind-matrix
deploy/loadtest/kind/compare.py --before <baseline dir> --after <patched dirs...>
```
