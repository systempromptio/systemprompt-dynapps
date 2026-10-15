# OCR Setup Guide

Two OCR backends are supported: **Ollama** (local, free) and **Z.AI Cloud** (hosted, API key).

## Option A: Ollama (Local)

### Installation

```bash
# macOS
brew install ollama

# Linux
curl -fsSL https://ollama.com/install.sh | sh

# Start the Ollama server
ollama serve
```

### Pull GLM-OCR Model

```bash
ollama pull glm-ocr
```

The `glm-ocr` model is a vision-language model optimized for document OCR. It accepts images and returns structured text.

### Configuration

```env
OCR_BACKEND="ollama"
OLLAMA_HOST="http://localhost:11434"
OLLAMA_MODEL="glm-ocr"
```

### Test Connection

```bash
curl http://localhost:11434/api/tags
```

Should list `glm-ocr` in the models array.

### API Endpoint

The skill uses the Ollama chat API:

```
POST {OLLAMA_HOST}/api/chat
```

Payload format:
```json
{
  "model": "glm-ocr",
  "messages": [
    {
      "role": "user",
      "content": "Extract document data as JSON...",
      "images": ["<base64-encoded-image>"]
    }
  ],
  "stream": false,
  "format": "json",
  "options": {
    "temperature": 0.1
  }
}
```

### Alternative Models

If `glm-ocr` is unavailable, these alternatives support vision + JSON output:

| Model | Size | Notes |
|---|---|---|
| `llava:13b` | 13B | Good general vision |
| `bakllava` | 7B | Lighter alternative |
| `minicpm-v` | 8B | Strong OCR capability |

Change the model in `.env`: `OLLAMA_MODEL="minicpm-v"`

## Option B: Z.AI Cloud

### Get API Key

1. Sign up at [open.bigmodel.cn](https://open.bigmodel.cn/)
2. Go to API Keys section
3. Create a new key

### Configuration

```env
OCR_BACKEND="zai"
ZAI_API_KEY="your-api-key-here"
ZAI_API_URL="https://open.bigmodel.cn/api/paas/v4/chat/completions"
```

### API Endpoint

The skill uses the Z.AI chat completions API with vision:

```
POST {ZAI_API_URL}
Authorization: Bearer {ZAI_API_KEY}
```

Payload format:
```json
{
  "model": "glm-4v-flash",
  "messages": [
    {
      "role": "user",
      "content": [
        {"type": "text", "text": "Extract document data as JSON..."},
        {"type": "image_url", "image_url": {"url": "data:image/png;base64,..."}}
      ]
    }
  ],
  "temperature": 0.1
}
```

## JSON Schema Prompt

Both backends receive the same structured extraction prompt. The model is asked to return JSON with these fields:

| Field | Type | Description |
|---|---|---|
| `document_type` | string | invoice, credit_note, delivery_note, purchase_order, expense_receipt, unknown |
| `vendor_name` | string | Vendor / supplier company name |
| `vendor_vat` | string | Vendor tax ID (CIF/NIF/VAT) |
| `vendor_address` | string | Vendor address |
| `customer_name` | string | Customer company name |
| `customer_vat` | string | Customer tax ID |
| `invoice_number` | string | Document reference number |
| `invoice_date` | string | Date in YYYY-MM-DD format |
| `due_date` | string | Payment due date |
| `currency` | string | ISO 4217 code (EUR, USD, GBP) |
| `lines` | array | Line items with description, quantity, unit_price, tax_percent, subtotal |
| `subtotal` | float | Net amount before tax |
| `tax_amount` | float | Total tax amount |
| `total` | float | Grand total |
| `payment_terms` | string | Payment terms text |
| `notes` | string | Additional notes |
| `raw_text` | string | All readable text from the document |

## Troubleshooting

| Issue | Solution |
|---|---|
| Ollama connection refused | Run `ollama serve` or check OLLAMA_HOST |
| Model not found | Run `ollama pull glm-ocr` |
| Z.AI 401 Unauthorized | Check ZAI_API_KEY in .env |
| Poor OCR quality | Increase DPI (edit `pdf_utils.py`, default 300) |
| Timeout on large pages | Increase timeout in `ocr_extract.py` (default 120s) |
| JSON parse errors | Model may need lower temperature or simpler prompt |
