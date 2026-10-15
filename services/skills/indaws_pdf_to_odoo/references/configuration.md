# Configuration reference

All settings live in `.env` (copy from `.env.example`). Nothing here is read at import time by
anything but `config.py`, so a wrong value shows up as behaviour, not as a crash — which is why
the defaults matter as much as the names.

## Odoo connection

| Variable | Required | Description |
|----------|----------|-------------|
| `ODOO_URL` | Yes | Odoo instance URL |
| `ODOO_DB` | Yes | Database name |
| `ODOO_KEY` | Yes | API key |
| `ODOO_USER` | No | API user (default: `api`) |

The connection contract itself — which protocol per version, MCP vs odoo-pilot, how to
authenticate — is `references/shared/odoo-connection.md`. This skill does not invent its
own client.

## Folder paths

| Variable | Default | Description |
|----------|---------|-------------|
| `PDF_INBOX_DIR` | `inbox` | Folder to watch for new PDFs |
| `PDF_ARCHIVE_DIR` | `archive` | Processed PDFs moved here |
| `PDF_ERRORS_DIR` | `errors` | Failed PDFs moved here |

## OCR backend

| Variable | Default | Description |
|----------|---------|-------------|
| `OCR_BACKEND` | `ollama` | OCR engine: `ollama` or `zai` |
| `OLLAMA_HOST` | `http://localhost:11434` | Ollama server URL |
| `OLLAMA_MODEL` | `glm-ocr` | Ollama vision model |
| `ZAI_API_KEY` | -- | Z.AI Cloud API key |
| `ZAI_API_URL` | `https://open.bigmodel.cn/...` | Z.AI endpoint |

**`zai` sends the document image to a third party.** That is a data-processing decision, not a
performance one: client invoices carry names, addresses and tax ids. Default to `ollama` unless
someone with authority to make that call decided otherwise.

## Processing options

| Variable | Default | Description |
|----------|---------|-------------|
| `PARTNER_MATCH_THRESHOLD` | `0.7` | Fuzzy name match cutoff (0.0-1.0) |
| `PARTNER_AUTO_CREATE` | `true` | Create partner if not found |
| `TRY_LINE_ITEMS` | `true` | Extract individual line items |
| `MATCH_PRODUCTS` | `false` | Match products by reference code |
| `SMART_SPLIT` | `true` | Detect multi-document PDFs |
| `DEFAULT_DOC_TYPE` | `in_invoice` | Fallback when classification fails |

The three defaults that decide how much mess a bad OCR can make are
`PARTNER_AUTO_CREATE=true`, `PARTNER_MATCH_THRESHOLD=0.7` and `DEFAULT_DOC_TYPE=in_invoice` — see
GOTCHAS in `SKILL.md` before leaving them as they come.

## Notifications

| Variable | Default | Description |
|----------|---------|-------------|
| `NOTIFY_EMAIL_TO` | -- | Email for error notifications |

Leaving it empty disables the mail entirely (`scripts/notify.py:25-26`, `send_error_notification`), which is the right setting on any
instance where outgoing mail is not deliberately configured.

## Troubleshooting

| Issue | Solution |
|-------|----------|
| `poppler not found` | Install poppler: `brew install poppler` (macOS) or `apt install poppler-utils` |
| OCR returns empty JSON | Check OCR backend is running (`ollama serve` or Z.AI API key) |
| Partner not matched | Lower `PARTNER_MATCH_THRESHOLD` or check VAT format in Odoo |
| Record creation fails | Verify Odoo API key permissions and model access rights |
| PDF stuck in inbox | Check `pdf-to-odoo.log` for errors, verify file permissions |
| Multi-page invoice split incorrectly | Set `SMART_SPLIT=false` to treat entire PDF as one document |
