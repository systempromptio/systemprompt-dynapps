# SNAPSHOT.md format and hash algorithm

`SNAPSHOT.md` is the human-readable summary of the machine-zone inventory. It is auto-generated — always. Manual edits are flagged by the hash check (`snapshot_stale`).

## File location

`.ai-memory/inventory/SNAPSHOT.md` (as declared by `.schema-version:snapshot_path`).

## Structure

The file has three sections in this exact order:

1. **Header** — fixed banner, do not vary.
2. **Summary** — per-source stats (one block per entry in `snapshot_sources`).
3. **Integrity** — the SHA256 hash and generation timestamp.

### Example

```markdown
# SNAPSHOT — Machine-zone inventory

> AUTO-GENERATED. Do not edit by hand. Re-run `memory-check sync` to refresh.
> Layer 2 review flags manual edits as CRITICAL via the integrity hash below.

Generated: 2026-04-17T09:42:11Z
Generator: memory-check@0.1.0

## modules.yml

- Custom modules: 3
  - acme_sale_approval (v18.0.1.0.0)
  - acme_stock_barcode (v18.0.1.0.0)
  - acme_hr_custom (v18.0.1.1.0)
- Indaws Core modules: 2
  - indaws_base (v18.0.1.0.0)
  - indaws_sale (v18.0.1.0.0)
- OCA modules: 1
  - partner_firstname (oca/partner-contact)
- Studio modules: 0

## studio.yml

- Custom fields: 0
- Custom models: 0
- Server actions: 0
- Automated actions: 0
- View modifications: 0
- Reports: 0

## integrations.yml

- Integrations: 0

## Integrity

SHA256: 7a8f3b2c9d1e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a
Sources hashed (in order):
- inventory/modules.yml
- inventory/studio.yml
- inventory/integrations.yml
```

## Hash algorithm

The hash validates that the YAMLs have not been edited since the snapshot was generated. Algorithm (must mirror `normalize_bytes()` in `scripts/snapshot_gen.py`):

1. Read each file listed in `.schema-version:snapshot_sources`, in the declared order.
2. Open in **binary mode** (do not decode).
3. **Strip the UTF-8 BOM** (`\xef\xbb\xbf`) if present at the start of the file.
4. Normalize line endings: `\r\n` → `\n`, then any remaining `\r` → `\n`.
5. Concatenate bytes in order, with a single `\n` separator between files.
6. Compute SHA256 on the concatenated bytes.
7. Hex-encode lowercase.

```python
import hashlib

def normalize_bytes(raw: bytes) -> bytes:
    # Strip UTF-8 BOM (some Windows editors add it silently).
    if raw.startswith(b"\xef\xbb\xbf"):
        raw = raw[3:]
    # Normalize line endings: CRLF first, then lone CR.
    return raw.replace(b"\r\n", b"\n").replace(b"\r", b"\n")

h = hashlib.sha256()
for path in snapshot_sources:
    with open(path, "rb") as f:
        h.update(normalize_bytes(f.read()))
    h.update(b"\n")  # single separator between sources
digest = h.hexdigest()
```

Both steps are mandatory: skipping BOM removal or line-ending normalization makes Windows and Linux checkouts produce different hashes (see [gotchas.md](gotchas.md) §4).

**Do not hash `SNAPSHOT.md` itself** — it contains the hash. The hash covers only the sources.

## Reading the stored hash

Parse the line matching `^SHA256: ([0-9a-f]{64})$` in the `## Integrity` section. If absent or malformed, treat the snapshot as stale (finding `snapshot_stale`, critical).

## Counting conventions

Each section header in the Summary matches the YAML key literal (`modules.yml`, `studio.yml`, `integrations.yml`). Counts follow the YAML structure:

- `modules.yml`: one sub-line per bucket (`custom`, `core`, `oca`, `studio`), plus an indented bullet for each entry. Show `technical_name` and `version` when available; otherwise just the name.
- `studio.yml`: one sub-line per top-level list (`custom_fields`, `custom_models`, `server_actions`, `automated_actions`, `view_modifications`, `reports`). No per-entry bullets in MVP (would duplicate the YAML).
- `integrations.yml`: one sub-line with total count. No per-entry bullets in MVP.

## Determinism

Two runs on the same YAML contents must produce byte-identical `SNAPSHOT.md` (modulo the timestamp). This means:

- Entries are listed in the order they appear in the YAML (not re-sorted).
- `version` fields are shown verbatim (no normalization).
- The timestamp is the only source of non-determinism; it is written with second precision in UTC ISO 8601 (`YYYY-MM-DDTHH:MM:SSZ`).

If a caller needs a stable hash-of-snapshot for testing, compute it with the timestamp line stripped.

## When the snapshot does not exist yet

First-time run: `SNAPSHOT.md` is absent. This is NOT a drift condition — it is an empty slate. `sync` generates it; `check` reports `snapshot_stale` so the user is nudged to run `sync`.
