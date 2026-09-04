"""
Anthropic + OpenAI → GCP Vertex AI proxy
=========================================
Translates:
  • Claude Code's Anthropic  /v1/messages          → Vertex AI
  • opencode/Copilot's OpenAI /v1/chat/completions   → Vertex AI

Two backends, picked per-model via MODEL_ROUTES:
  • "genai" — Google's own Gemini models, called through the google-genai SDK
    (Application Default Credentials, no key needed).
  • "maas"  — Vertex AI's Model-as-a-Service OpenAI-compatible endpoint, used
    for third-party open models (GLM, Kimi, Qwen, DeepSeek, GPT-OSS, ...).
    Auth is a bearer token pulled fresh from the same ADC credentials.

Auth: run `gcloud auth application-default login` once; no API key is stored
or transmitted.

Usage:
  cd gcp-proxy
  python proxy.py
"""

import csv
import datetime
import json
import os
import random
import time
from pathlib import Path

import google.auth
import google.auth.transport.requests
import requests as http_requests
import uvicorn
from dotenv import load_dotenv
from fastapi import Depends, FastAPI, Header, HTTPException, Request, Response
from fastapi.responses import HTMLResponse, StreamingResponse
from google import genai
from google.genai import types

try:
    import psycopg2
    from psycopg2.extras import RealDictCursor

    _HAS_PG = True
except ImportError:
    _HAS_PG = False

# ── Config ────────────────────────────────────────────────────────────────────
BASE_DIR = Path(__file__).parent
load_dotenv(BASE_DIR / ".env.gcp")

GCP_PROJECT = os.getenv("GCP_PROJECT", "enterprise-ai-505512")
GCP_LOCATION = os.getenv("GCP_LOCATION", "us-central1")
DATABASE_URL = os.getenv(
    "DATABASE_URL", "postgresql://postgres:postgres@localhost:5432/litellm"
)


# ── Auth (only enforced once keys are configured) ──────────────────────────────
def _parse_api_keys(raw: str) -> dict[str, str]:
    keys = {}
    for entry in raw.split(","):
        entry = entry.strip()
        if not entry:
            continue
        if ":" in entry:
            name, key = entry.split(":", 1)
            keys[key.strip()] = name.strip()
        else:
            keys[entry] = entry
    return keys


PROXY_API_KEYS = _parse_api_keys(os.getenv("PROXY_API_KEYS", ""))


def require_api_key(authorization: str = Header(default="")) -> str:
    if not PROXY_API_KEYS:
        return "local"
    token = authorization.removeprefix("Bearer ").strip()
    label = PROXY_API_KEYS.get(token)
    if label is None:
        raise HTTPException(status_code=401, detail="Invalid or missing API key")
    return label


genai_client = genai.Client(vertexai=True, project=GCP_PROJECT, location=GCP_LOCATION)

MAAS_URL = (
    f"https://aiplatform.googleapis.com/v1beta1/projects/{GCP_PROJECT}"
    f"/locations/global/endpoints/openapi/chat/completions"
)

_credentials, _ = google.auth.default()


def get_access_token() -> str:
    if not _credentials.valid:
        _credentials.refresh(google.auth.transport.requests.Request())
    return _credentials.token


# ── Retry / backoff config ──────────────────────────────────────────────────
# Vertex AI's MaaS endpoints (and occasionally the Gemini/genai backend) return
# 429 under per-model RPM quota or bursty traffic. Rather than surface that as
# an error (which is what was happening — OpenCode saw a hard failure and
# stopped), every backend call below retries transparently with exponential
# backoff + jitter until it succeeds. While backing off, streaming calls emit
# an SSE keep-alive so the client connection never times out and OpenCode
# never needs a restart.
RETRY_INITIAL_SECONDS = 1.0
RETRY_MAX_SECONDS = 30.0
RETRY_MULTIPLIER = 2.0
RETRY_JITTER_FRACTION = 0.25

try:
    from google.api_core.exceptions import GoogleAPICallError
except (
    ImportError
):  # pragma: no cover — google-api-core always ships with genai, but stay defensive
    GoogleAPICallError = None


def _next_backoff(attempt: int) -> float:
    base = min(RETRY_INITIAL_SECONDS * (RETRY_MULTIPLIER**attempt), RETRY_MAX_SECONDS)
    return base + random.uniform(0, base * RETRY_JITTER_FRACTION)


def _is_retryable_status(status_code: int) -> bool:
    return status_code == 429 or status_code >= 500


def _is_retryable_genai_error(exc: Exception) -> bool:
    if GoogleAPICallError is not None and isinstance(exc, GoogleAPICallError):
        code = getattr(exc, "code", None)
        status_code = getattr(code, "value", code) if code is not None else None
        if status_code in (429, 503):
            return True
    msg = str(exc).upper()
    return any(
        s in msg
        for s in ("429", "RESOURCE_EXHAUSTED", "RATE LIMIT", "QUOTA", "UNAVAILABLE")
    )


# ── Model routing table ───────────────────────────────────────────────────────
MODEL_ROUTES = {
    "gemini-2.5-pro": {
        "backend": "genai",
        "vertex_model": "gemini-2.5-pro",
        "input_cost": 1.25 / 1_000_000,
        "output_cost": 10.00 / 1_000_000,
        "label": "Gemini 2.5 Pro",
    },
    "gemini-2.5-flash": {
        "backend": "genai",
        "vertex_model": "gemini-2.5-flash",
        "input_cost": 0.30 / 1_000_000,
        "output_cost": 2.50 / 1_000_000,
        "label": "Gemini 2.5 Flash",
    },
    "gemini-2.5-flash-lite": {
        "backend": "genai",
        "vertex_model": "gemini-2.5-flash-lite",
        "input_cost": 0.10 / 1_000_000,
        "output_cost": 0.40 / 1_000_000,
        "label": "Gemini 2.5 Flash-Lite",
    },
    "zai.glm-5": {
        "backend": "maas",
        "vertex_model": "zai-org/glm-5-maas",
        "input_cost": 1.00 / 1_000_000,
        "output_cost": 3.20 / 1_000_000,
        "label": "GLM-5",
    },
    "moonshotai.kimi-k2-thinking": {
        "backend": "maas",
        "vertex_model": "moonshotai/kimi-k2-thinking-maas",
        "input_cost": 0.60 / 1_000_000,
        "output_cost": 2.50 / 1_000_000,
        "label": "Kimi K2 Thinking",
        "reasoning": True,
    },
    "qwen.qwen3-next-thinking": {
        "backend": "maas",
        "vertex_model": "qwen/qwen3-next-80b-a3b-thinking-maas",
        "input_cost": 0.15 / 1_000_000,
        "output_cost": 1.20 / 1_000_000,
        "label": "Qwen3-Next-80B Thinking",
        "reasoning": True,
    },
    "qwen.qwen3-next-instruct": {
        "backend": "maas",
        "vertex_model": "qwen/qwen3-next-80b-a3b-instruct-maas",
        "input_cost": 0.15 / 1_000_000,
        "output_cost": 1.20 / 1_000_000,
        "label": "Qwen3-Next-80B Instruct",
    },
    "qwen.qwen3-coder-480b": {
        "backend": "maas",
        "vertex_model": "qwen/qwen3-coder-480b-a35b-instruct-maas",
        "input_cost": 1.00 / 1_000_000,
        "output_cost": 4.00 / 1_000_000,
        "label": "Qwen3-Coder-480B (flagship coder)",
    },
    "qwen.qwen3-235b": {
        "backend": "maas",
        "vertex_model": "qwen/qwen3-235b-a22b-instruct-2507-maas",
        "input_cost": 0.25 / 1_000_000,
        "output_cost": 1.00 / 1_000_000,
        "label": "Qwen3-235B-A22B",
    },
    "deepseek.v3.2": {
        "backend": "maas",
        "vertex_model": "deepseek-ai/deepseek-v3.2-maas",
        "input_cost": 0.56 / 1_000_000,
        "output_cost": 1.68 / 1_000_000,
        "label": "DeepSeek V3.2",
    },
    "openai.gpt-oss-120b": {
        "backend": "maas",
        "vertex_model": "openai/gpt-oss-120b-maas",
        "input_cost": 0.15 / 1_000_000,
        "output_cost": 0.60 / 1_000_000,
        "label": "GPT-OSS-120B",
    },
    "openai.gpt-oss-20b": {
        "backend": "maas",
        "vertex_model": "openai/gpt-oss-20b-maas",
        "input_cost": 0.07 / 1_000_000,
        "output_cost": 0.30 / 1_000_000,
        "label": "GPT-OSS-20B",
    },
}
DEFAULT_MODEL = "qwen.qwen3-next-instruct"

print(f"✅  Vertex project : {GCP_PROJECT}")
print(f"✅  Auth           : Application Default Credentials")
print(f"✅  Models         :")
for alias, info in MODEL_ROUTES.items():
    print(f"     {alias:32s} → {info['label']}")
print()

session_stats = {
    "requests": 0,
    "input_tokens": 0,
    "output_tokens": 0,
    "total_cost_usd": 0.0,
}

app = FastAPI()

_DB_OK = False

# ── CSV / PostgreSQL logging ───────────────────────────────────────────────────


def log_to_csv(
    timestamp,
    client_name,
    model_name,
    backend_model,
    prompt,
    completion,
    input_tokens,
    output_tokens,
    cost,
):
    csv_file = BASE_DIR / "request_logs.csv"
    file_exists = csv_file.exists()
    with open(csv_file, "a", newline="", encoding="utf-8") as f:
        writer = csv.writer(f)
        if not file_exists:
            writer.writerow(
                [
                    "timestamp",
                    "client",
                    "model_name",
                    "backend_model",
                    "input_tokens",
                    "output_tokens",
                    "cost_usd",
                    "prompt",
                    "completion",
                ]
            )
        writer.writerow(
            [
                timestamp.isoformat(),
                client_name,
                model_name,
                backend_model,
                input_tokens,
                output_tokens,
                round(cost, 8),
                prompt[:1000],
                completion[:1000],
            ]
        )


def _get_conn():
    if not _HAS_PG:
        return None
    try:
        return psycopg2.connect(DATABASE_URL, connect_timeout=3)
    except Exception as e:
        print(f"⚠️   DB connect failed: {str(e)[:80]}")
        return None


def init_db():
    global _DB_OK
    conn = _get_conn()
    if conn is None:
        print(
            "⚠️   PostgreSQL not available — logging to CSV only, dashboard shows session stats."
        )
        return
    try:
        with conn, conn.cursor() as cur:
            cur.execute("""
                CREATE TABLE IF NOT EXISTS token_usage_logs (
                    id             SERIAL PRIMARY KEY,
                    ts             TIMESTAMPTZ  NOT NULL DEFAULT now(),
                    client         TEXT,
                    model_name     TEXT,
                    backend_model  TEXT,
                    prompt         TEXT,
                    completion     TEXT,
                    input_tokens   INTEGER,
                    output_tokens  INTEGER,
                    cost_usd       NUMERIC(12, 8)
                );
            """)
        _DB_OK = True
        print("✅  PostgreSQL logging enabled → token_usage_logs")
        print("✅  CSV logging enabled → request_logs.csv")
    except Exception as e:
        print(f"⚠️   DB init failed: {str(e)[:80]}")
    finally:
        conn.close()


def log_to_db(
    client_name,
    model_name,
    backend_model,
    prompt,
    completion,
    input_tokens,
    output_tokens,
    cost,
):
    if not _DB_OK:
        return
    conn = _get_conn()
    if conn is None:
        return
    try:
        with conn, conn.cursor() as cur:
            cur.execute(
                """
                INSERT INTO token_usage_logs
                    (client, model_name, backend_model, prompt, completion,
                     input_tokens, output_tokens, cost_usd)
                VALUES (%s, %s, %s, %s, %s, %s, %s, %s)
                """,
                (
                    client_name,
                    model_name,
                    backend_model,
                    prompt,
                    completion,
                    input_tokens,
                    output_tokens,
                    cost,
                ),
            )
    except Exception as e:
        print(f"⚠️   DB insert failed: {str(e)[:80]}")
    finally:
        conn.close()


def fetch_recent_logs(limit=50):
    if not _DB_OK:
        return []
    conn = _get_conn()
    if conn is None:
        return []
    try:
        with conn, conn.cursor(cursor_factory=RealDictCursor) as cur:
            cur.execute(
                """
                SELECT ts, client, model_name, prompt, completion,
                       input_tokens, output_tokens, cost_usd
                FROM token_usage_logs ORDER BY id DESC LIMIT %s
                """,
                (limit,),
            )
            return cur.fetchall()
    except Exception as e:
        print(f"⚠️   DB fetch failed: {str(e)[:80]}")
        return []
    finally:
        conn.close()


# ── Request helpers ────────────────────────────────────────────────────────────


def resolve_route(requested_model: str) -> dict:
    return MODEL_ROUTES.get(requested_model, MODEL_ROUTES[DEFAULT_MODEL])


REASONING_MIN_TOKENS = 512


def apply_reasoning_floor(route: dict, max_tokens: int) -> int:
    if route.get("reasoning") and max_tokens < REASONING_MIN_TOKENS:
        return REASONING_MIN_TOKENS
    return max_tokens


def extract_prompt_text(messages: list) -> str:
    return "\n".join(
        f"[{m.get('role', '?')}] {m.get('content') or ''}" for m in messages
    )


def flatten_content(content) -> str:
    if isinstance(content, list):
        parts = []
        for b in content:
            if b.get("type") == "text":
                parts.append(b.get("text", ""))
            elif b.get("type") == "tool_result":
                parts.append(flatten_content(b.get("content", "")))
        return " ".join(parts)
    return content or ""


def extract_system(system) -> str | None:
    if isinstance(system, str) and system:
        return system
    if isinstance(system, list):
        text = " ".join(b.get("text", "") for b in system if b.get("type") == "text")
        return text or None
    return None


# ── Tool-calling format conversion ──────────────────────────────────────────────


def anthropic_tools_to_openai(tools):
    if not tools:
        return None
    return [
        {
            "type": "function",
            "function": {
                "name": t["name"],
                "description": t.get("description", ""),
                "parameters": t.get("input_schema")
                or {"type": "object", "properties": {}},
            },
        }
        for t in tools
    ]


def anthropic_tool_choice_to_openai(tool_choice):
    if not tool_choice:
        return None
    t = tool_choice.get("type")
    if t == "none":
        return "none"
    if t == "auto":
        return "auto"
    if t == "any":
        return "required"
    if t == "tool":
        name = tool_choice.get("name")
        if name:
            return {"type": "function", "function": {"name": name}}
    return None


def anthropic_messages_to_openai(messages: list):
    out = []
    for m in messages:
        role = m["role"]
        content = m["content"]
        if isinstance(content, str):
            out.append({"role": role, "content": content})
            continue

        text_parts, tool_calls, tool_results = [], [], []
        for b in content:
            t = b.get("type")
            if t == "text":
                text_parts.append(b.get("text", ""))
            elif t == "tool_use":
                tool_calls.append(
                    {
                        "id": b["id"],
                        "type": "function",
                        "function": {
                            "name": b["name"],
                            "arguments": json.dumps(b.get("input", {})),
                        },
                    }
                )
            elif t == "tool_result":
                tool_results.append(
                    {
                        "role": "tool",
                        "tool_call_id": b["tool_use_id"],
                        "content": flatten_content(b.get("content", "")),
                    }
                )

        if tool_calls:
            out.append(
                {
                    "role": "assistant",
                    "content": " ".join(text_parts) or None,
                    "tool_calls": tool_calls,
                }
            )
        elif text_parts:
            out.append({"role": role, "content": " ".join(text_parts)})
        out.extend(tool_results)
    return out


def openai_tool_calls_to_anthropic_blocks(text, tool_calls):
    blocks = []
    if text:
        blocks.append({"type": "text", "text": text})
    for i, tc in enumerate(tool_calls or []):
        fn = tc.get("function", {})
        try:
            input_obj = json.loads(fn.get("arguments") or "{}")
        except json.JSONDecodeError:
            input_obj = {}
        blocks.append(
            {
                "type": "tool_use",
                "id": tc.get("id") or f"toolu_{i}",
                "name": fn.get("name", ""),
                "input": input_obj,
            }
        )
    return blocks or [{"type": "text", "text": ""}]


# ── genai backend (Gemini) ─────────────────────────────────────────────────────


def clean_schema_for_gemini(schema):
    if not isinstance(schema, dict):
        return schema
    banned = {
        "$schema",
        "$id",
        "$ref",
        "$comment",
        "default",
        "exclusiveMinimum",
        "exclusiveMaximum",
        "examples",
        "contentMediaType",
        "contentEncoding",
        "if",
        "then",
        "else",
        "allOf",
        "anyOf",
        "oneOf",
        "not",
    }
    cleaned = {}
    for k, v in schema.items():
        if k in banned:
            continue
        if isinstance(v, dict):
            cleaned[k] = clean_schema_for_gemini(v)
        elif isinstance(v, list):
            cleaned[k] = [clean_schema_for_gemini(item) for item in v]
        else:
            cleaned[k] = v
    return cleaned


def openai_tools_to_genai(tools):
    if not tools:
        return None
    decls = []
    for t in tools:
        fn = t.get("function", t)
        params = fn.get("parameters") or {"type": "object", "properties": {}}
        decls.append(
            types.FunctionDeclaration(
                name=fn.get("name", ""),
                description=fn.get("description") or "",
                parameters=clean_schema_for_gemini(params),
            )
        )
    return [types.Tool(function_declarations=decls)]


def openai_tool_choice_to_genai(tool_choice):
    if not tool_choice:
        return None
    if tool_choice == "none":
        return types.ToolConfig(
            function_calling_config=types.FunctionCallingConfig(mode="NONE")
        )
    if tool_choice == "required":
        return types.ToolConfig(
            function_calling_config=types.FunctionCallingConfig(mode="ANY")
        )
    if tool_choice == "auto":
        return types.ToolConfig(
            function_calling_config=types.FunctionCallingConfig(mode="AUTO")
        )
    if isinstance(tool_choice, dict) and tool_choice.get("type") == "function":
        name = tool_choice.get("function", {}).get("name")
        return types.ToolConfig(
            function_calling_config=types.FunctionCallingConfig(
                mode="ANY",
                allowed_function_names=[name] if name else None,
            )
        )
    return None


def messages_to_genai(messages: list):
    contents = []
    id_to_name = {}
    pending_tool_parts = []

    def flush_tool_parts():
        if pending_tool_parts:
            contents.append(types.Content(role="user", parts=list(pending_tool_parts)))
            pending_tool_parts.clear()

    for m in messages:
        role = m["role"]

        if role == "tool":
            name = id_to_name.get(m.get("tool_call_id"), m.get("name", ""))
            pending_tool_parts.append(
                types.Part.from_function_response(
                    name=name, response={"result": m.get("content") or ""}
                )
            )
            continue
        flush_tool_parts()

        if role == "assistant" and m.get("tool_calls"):
            parts = []
            if m.get("content"):
                parts.append(types.Part(text=m["content"]))
            for tc in m["tool_calls"]:
                fn = tc.get("function", {})
                name = fn.get("name", "")
                try:
                    args = json.loads(fn.get("arguments") or "{}")
                except json.JSONDecodeError:
                    args = {}
                id_to_name[tc.get("id")] = name
                parts.append(types.Part.from_function_call(name=name, args=args))
            contents.append(types.Content(role="model", parts=parts))
            continue

        genai_role = "model" if role == "assistant" else "user"
        contents.append(
            types.Content(
                role=genai_role, parts=[types.Part(text=m.get("content") or "")]
            )
        )
    flush_tool_parts()
    return contents


def _genai_parts_to_response(candidate):
    text_parts, tool_calls = [], []
    if candidate and candidate.content and candidate.content.parts:
        for part in candidate.content.parts:
            fc = getattr(part, "function_call", None)
            if fc:
                tool_calls.append(
                    {
                        "id": f"call_{len(tool_calls)}",
                        "type": "function",
                        "function": {
                            "name": fc.name,
                            "arguments": json.dumps(dict(fc.args) if fc.args else {}),
                        },
                    }
                )
            elif getattr(part, "text", None):
                text_parts.append(part.text)
    return "".join(text_parts), (tool_calls or None)


def call_vertex(
    vertex_model,
    system_instruction,
    messages,
    max_tokens,
    temperature,
    tools=None,
    tool_choice=None,
):
    config = types.GenerateContentConfig(
        system_instruction=system_instruction,
        max_output_tokens=max_tokens,
        temperature=temperature,
        tools=openai_tools_to_genai(tools),
        tool_config=openai_tool_choice_to_genai(tool_choice),
    )
    attempt = 0
    while True:
        try:
            response = genai_client.models.generate_content(
                model=vertex_model, contents=messages_to_genai(messages), config=config
            )
            break
        except Exception as e:
            if not _is_retryable_genai_error(e):
                raise
            wait = _next_backoff(attempt)
            attempt += 1
            print(
                f"⏳  Gemini rate-limited on attempt {attempt} — retrying in {wait:.1f}s...",
                flush=True,
            )
            time.sleep(wait)

    candidate = response.candidates[0] if response.candidates else None
    text, tool_calls = _genai_parts_to_response(candidate)
    usage = response.usage_metadata
    return (
        text,
        tool_calls,
        (usage.prompt_token_count if usage else 0),
        (usage.candidates_token_count if usage else 0),
    )


def call_vertex_stream(
    vertex_model,
    system_instruction,
    messages,
    max_tokens,
    temperature,
    tools=None,
    tool_choice=None,
):
    config = types.GenerateContentConfig(
        system_instruction=system_instruction,
        max_output_tokens=max_tokens,
        temperature=temperature,
        tools=openai_tools_to_genai(tools),
        tool_config=openai_tool_choice_to_genai(tool_choice),
    )
    attempt = 0
    while True:
        input_tokens = output_tokens = 0
        tool_calls = []
        started_output = False
        try:
            for chunk in genai_client.models.generate_content_stream(
                model=vertex_model, contents=messages_to_genai(messages), config=config
            ):
                candidate = chunk.candidates[0] if chunk.candidates else None
                text, chunk_tool_calls = _genai_parts_to_response(candidate)
                if text:
                    started_output = True
                    yield ("text", text)
                if chunk_tool_calls:
                    started_output = True
                    for tc in chunk_tool_calls:
                        tc["id"] = f"call_{len(tool_calls)}"
                        tool_calls.append(tc)
                if chunk.usage_metadata:
                    input_tokens = (
                        chunk.usage_metadata.prompt_token_count or input_tokens
                    )
                    output_tokens = (
                        chunk.usage_metadata.candidates_token_count or output_tokens
                    )
            if tool_calls:
                yield ("tool_calls", tool_calls)
            yield ("usage", (input_tokens, output_tokens))
            return
        except Exception as e:
            # Only safe to retry the whole generation from scratch if nothing
            # has reached the client yet — otherwise we'd duplicate output.
            if started_output or not _is_retryable_genai_error(e):
                raise
            wait = _next_backoff(attempt)
            attempt += 1
            print(
                f"⏳  Gemini stream rate-limited on attempt {attempt} — retrying in {wait:.1f}s...",
                flush=True,
            )
            yield ("retry", wait)
            time.sleep(wait)
            continue


# ── maas backend (GLM / Kimi / Qwen / DeepSeek / GPT-OSS) ──────────────────────


def _maas_messages(system_instruction, messages):
    out = []
    if system_instruction:
        out.append({"role": "system", "content": system_instruction})
    for m in messages:
        entry = {"role": m["role"], "content": m.get("content")}
        if m.get("tool_calls"):
            entry["tool_calls"] = m["tool_calls"]
        if m.get("tool_call_id"):
            entry["tool_call_id"] = m["tool_call_id"]
        out.append(entry)
    return out


def _maas_payload(
    vertex_model,
    system_instruction,
    messages,
    max_tokens,
    temperature,
    stream,
    tools,
    tool_choice,
):
    payload = {
        "model": vertex_model,
        "messages": _maas_messages(system_instruction, messages),
        "max_tokens": max_tokens,
        "temperature": temperature,
        "stream": stream,
    }
    if tools:
        payload["tools"] = tools
    if tool_choice:
        payload["tool_choice"] = tool_choice
    return payload


def call_maas(
    vertex_model,
    system_instruction,
    messages,
    max_tokens,
    temperature,
    tools=None,
    tool_choice=None,
):
    attempt = 0
    while True:
        resp = http_requests.post(
            MAAS_URL,
            headers={
                "Authorization": f"Bearer {get_access_token()}",
                "Content-Type": "application/json",
                "X-Goog-User-Project": GCP_PROJECT,
            },
            json=_maas_payload(
                vertex_model,
                system_instruction,
                messages,
                max_tokens,
                temperature,
                False,
                tools,
                tool_choice,
            ),
            timeout=120,
        )
        if resp.status_code < 400:
            break
        if _is_retryable_status(resp.status_code):
            wait = _next_backoff(attempt)
            attempt += 1
            print(
                f"⏳  MaaS {resp.status_code} on attempt {attempt} — retrying in {wait:.1f}s...",
                flush=True,
            )
            resp.close()
            time.sleep(wait)
            continue
        resp.raise_for_status()  # non-retryable (4xx other than 429) — raise as before

    data = resp.json()
    msg = data["choices"][0]["message"]
    text = msg.get("content") or ""
    tool_calls = msg.get("tool_calls") or None
    usage = data.get("usage", {})
    return (
        text,
        tool_calls,
        usage.get("prompt_tokens", 0),
        usage.get("completion_tokens", 0),
    )


def call_maas_stream(
    vertex_model,
    system_instruction,
    messages,
    max_tokens,
    temperature,
    tools=None,
    tool_choice=None,
):
    attempt = 0
    while True:
        resp = http_requests.post(
            MAAS_URL,
            headers={
                "Authorization": f"Bearer {get_access_token()}",
                "Content-Type": "application/json",
                "X-Goog-User-Project": GCP_PROJECT,
            },
            json=_maas_payload(
                vertex_model,
                system_instruction,
                messages,
                max_tokens,
                temperature,
                True,
                tools,
                tool_choice,
            ),
            stream=True,
            timeout=120,
        )
        if resp.status_code < 400:
            break
        if _is_retryable_status(resp.status_code):
            wait = _next_backoff(attempt)
            attempt += 1
            print(
                f"⏳  MaaS stream {resp.status_code} on attempt {attempt} — retrying in {wait:.1f}s...",
                flush=True,
            )
            resp.close()
            yield ("retry", wait)
            time.sleep(wait)
            continue
        resp.raise_for_status()

    input_tokens = output_tokens = 0
    tool_call_acc = {}
    for raw_line in resp.iter_lines():
        if not raw_line:
            continue
        line = (
            raw_line.decode("utf-8") if isinstance(raw_line, bytes) else str(raw_line)
        )
        if not line.startswith("data: "):
            continue
        data = line[6:]
        if data == "[DONE]":
            break
        try:
            chunk = json.loads(data)
        except json.JSONDecodeError:
            continue
        choices = chunk.get("choices") or []
        delta = choices[0].get("delta", {}) if choices else {}
        text_piece = delta.get("content") or ""
        for tc_delta in delta.get("tool_calls") or []:
            idx = tc_delta.get("index", 0)
            entry = tool_call_acc.setdefault(
                idx,
                {
                    "id": "",
                    "type": "function",
                    "function": {"name": "", "arguments": ""},
                },
            )
            if tc_delta.get("id"):
                entry["id"] = tc_delta["id"]
            fn_delta = tc_delta.get("function") or {}
            if fn_delta.get("name"):
                entry["function"]["name"] += fn_delta["name"]
            if fn_delta.get("arguments"):
                entry["function"]["arguments"] += fn_delta["arguments"]
        u = chunk.get("usage") or {}
        if u.get("prompt_tokens"):
            input_tokens = u["prompt_tokens"]
        if u.get("completion_tokens"):
            output_tokens = u["completion_tokens"]
        if text_piece:
            yield ("text", text_piece)
    if tool_call_acc:
        yield ("tool_calls", [tool_call_acc[i] for i in sorted(tool_call_acc)])
    yield ("usage", (input_tokens, output_tokens))


# ── Backend-agnostic dispatch ───────────────────────────────────────────────────


def generate_once(
    route,
    system_instruction,
    messages,
    max_tokens,
    temperature,
    tools=None,
    tool_choice=None,
):
    fn = call_vertex if route["backend"] == "genai" else call_maas
    return fn(
        route["vertex_model"],
        system_instruction,
        messages,
        max_tokens,
        temperature,
        tools=tools,
        tool_choice=tool_choice,
    )


def generate_stream(
    route,
    system_instruction,
    messages,
    max_tokens,
    temperature,
    tools=None,
    tool_choice=None,
):
    fn = call_vertex_stream if route["backend"] == "genai" else call_maas_stream
    yield from fn(
        route["vertex_model"],
        system_instruction,
        messages,
        max_tokens,
        temperature,
        tools=tools,
        tool_choice=tool_choice,
    )


def log_cost(
    input_tokens,
    output_tokens,
    route,
    requested_model="",
    client_name="",
    prompt="",
    completion="",
):
    cost = (input_tokens * route["input_cost"]) + (output_tokens * route["output_cost"])
    session_stats["requests"] += 1
    session_stats["input_tokens"] += input_tokens
    session_stats["output_tokens"] += output_tokens
    session_stats["total_cost_usd"] += cost

    print(
        f"📊  Request #{session_stats['requests']:>4}  [{route['label']:28s}]  {client_name:>6}  |  "
        f"in={input_tokens:>6} out={output_tokens:>6}  |  "
        f"cost=${cost:.6f}  |  session total=${session_stats['total_cost_usd']:.6f}"
    )

    timestamp = datetime.datetime.now()
    log_to_csv(
        timestamp,
        client_name,
        requested_model or route["label"],
        route["vertex_model"],
        prompt,
        completion,
        input_tokens,
        output_tokens,
        cost,
    )
    log_to_db(
        client_name,
        requested_model or route["label"],
        route["vertex_model"],
        prompt,
        completion,
        input_tokens,
        output_tokens,
        round(cost, 8),
    )
    return cost


# ── Routes: Anthropic (Claude Code) ────────────────────────────────────────────


@app.post("/v1/messages")
@app.post("/anthropic/v1/messages")
async def messages(request: Request, caller: str = Depends(require_api_key)):
    body = await request.json()
    is_stream = body.get("stream", False)
    requested_model = body.get("model", DEFAULT_MODEL)
    route = resolve_route(requested_model)

    raw_messages = anthropic_messages_to_openai(body.get("messages", []))
    tools = anthropic_tools_to_openai(body.get("tools"))
    anthropic_tool_choice = body.get("tool_choice")
    system_instruction = extract_system(body.get("system"))
    max_tokens = apply_reasoning_floor(route, body.get("max_tokens", 1024))
    temperature = body.get("temperature", 1.0)
    prompt_text = extract_prompt_text(raw_messages)

    if is_stream:

        def generate():
            full_text, input_tokens, output_tokens, tool_calls = [], 0, 0, None
            try:
                yield f"data: {json.dumps({'type': 'message_start', 'message': {'id': 'msg_proxy', 'type': 'message', 'role': 'assistant', 'content': [], 'model': requested_model, 'usage': {'input_tokens': 0, 'output_tokens': 0}}})}\n\n"
                yield f"data: {json.dumps({'type': 'content_block_start', 'index': 0, 'content_block': {'type': 'text', 'text': ''}})}\n\n"
                yield f"data: {json.dumps({'type': 'ping'})}\n\n"

                for kind, payload in generate_stream(
                    route,
                    system_instruction,
                    raw_messages,
                    max_tokens,
                    temperature,
                    tools=tools,
                    tool_choice=anthropic_tool_choice_to_openai(anthropic_tool_choice),
                ):
                    if kind == "text":
                        full_text.append(payload)
                        yield f"data: {json.dumps({'type': 'content_block_delta', 'index': 0, 'delta': {'type': 'text_delta', 'text': payload}})}\n\n"
                    elif kind == "tool_calls":
                        tool_calls = payload
                    elif kind == "retry":
                        # Native Anthropic SSE ping — keeps the connection alive
                        # while we back off and retry the upstream 429/5xx.
                        yield f"data: {json.dumps({'type': 'ping'})}\n\n"
                    else:
                        input_tokens, output_tokens = payload
            except Exception as e:
                err = str(e)[:500]
                print(f"❌  Vertex error: {err}", flush=True)
                import traceback

                traceback.print_exc()
                yield f"data: {json.dumps({'type': 'error', 'error': {'type': 'api_error', 'message': err}})}\n\n"
                return

            log_cost(
                input_tokens,
                output_tokens,
                route,
                requested_model,
                caller,
                prompt_text,
                "".join(full_text),
            )
            yield f"data: {json.dumps({'type': 'content_block_stop', 'index': 0})}\n\n"

            index = 1
            for tc in tool_calls or []:
                fn = tc.get("function", {})
                try:
                    input_obj = json.loads(fn.get("arguments") or "{}")
                except json.JSONDecodeError:
                    input_obj = {}
                tool_id = tc.get("id") or f"toolu_{index}"
                yield f"data: {json.dumps({'type': 'content_block_start', 'index': index, 'content_block': {'type': 'tool_use', 'id': tool_id, 'name': fn.get('name', ''), 'input': {}}})}\n\n"
                yield f"data: {json.dumps({'type': 'content_block_delta', 'index': index, 'delta': {'type': 'input_json_delta', 'partial_json': json.dumps(input_obj)}})}\n\n"
                yield f"data: {json.dumps({'type': 'content_block_stop', 'index': index})}\n\n"
                index += 1

            stop_reason = "tool_use" if tool_calls else "end_turn"
            yield f"data: {json.dumps({'type': 'message_delta', 'delta': {'stop_reason': stop_reason, 'stop_sequence': None}, 'usage': {'output_tokens': output_tokens}})}\n\n"
            yield f"data: {json.dumps({'type': 'message_stop'})}\n\n"

        return StreamingResponse(generate(), media_type="text/event-stream")

    try:
        text, tool_calls, input_tokens, output_tokens = generate_once(
            route,
            system_instruction,
            raw_messages,
            max_tokens,
            temperature,
            tools=tools,
            tool_choice=anthropic_tool_choice_to_openai(anthropic_tool_choice),
        )
    except Exception as e:
        err = str(e)[:500]
        print(f"❌  Vertex error: {err}", flush=True)
        import traceback

        traceback.print_exc()
        return Response(
            content=json.dumps({"error": {"type": "api_error", "message": err}}),
            status_code=502,
            media_type="application/json",
        )

    log_cost(
        input_tokens, output_tokens, route, requested_model, caller, prompt_text, text
    )
    return {
        "id": "msg_proxy",
        "type": "message",
        "role": "assistant",
        "content": openai_tool_calls_to_anthropic_blocks(text, tool_calls),
        "model": requested_model,
        "stop_reason": "tool_use" if tool_calls else "end_turn",
        "stop_sequence": None,
        "usage": {"input_tokens": input_tokens, "output_tokens": output_tokens},
    }


# ── Routes: OpenAI (opencode, VS Code Copilot BYOK, Aider) ─────────────────────


@app.post("/v1/chat/completions")
async def chat_completions(request: Request, caller: str = Depends(require_api_key)):
    body = await request.json()
    is_stream = body.get("stream", False)
    requested_model = body.get("model", DEFAULT_MODEL)
    route = resolve_route(requested_model)

    system_instruction = None
    raw_messages = []
    for m in body.get("messages", []):
        content = (
            flatten_content(m.get("content")) if m.get("content") is not None else None
        )
        if m["role"] == "system" and system_instruction is None:
            system_instruction = content or None
            continue
        entry = {"role": m["role"], "content": content}
        if m.get("tool_calls"):
            entry["tool_calls"] = m["tool_calls"]
        if m.get("tool_call_id"):
            entry["tool_call_id"] = m["tool_call_id"]
        raw_messages.append(entry)

    tools = body.get("tools")
    tool_choice = body.get("tool_choice")
    max_tokens = apply_reasoning_floor(route, body.get("max_tokens", 1024))
    temperature = body.get("temperature", 1.0)
    prompt_text = extract_prompt_text(raw_messages)

    if is_stream:

        def generate():
            full_text, input_tokens, output_tokens, tool_calls = [], 0, 0, None
            created = int(datetime.datetime.now().timestamp())

            yield f"data: {json.dumps({'id': 'chatcmpl-proxy', 'object': 'chat.completion.chunk', 'created': created, 'model': requested_model, 'system_fingerprint': None, 'choices': [{'index': 0, 'delta': {'role': 'assistant'}, 'logprobs': None, 'finish_reason': None}]})}\n\n"

            try:
                for kind, payload in generate_stream(
                    route,
                    system_instruction,
                    raw_messages,
                    max_tokens,
                    temperature,
                    tools=tools,
                    tool_choice=tool_choice,
                ):
                    if kind == "text":
                        full_text.append(payload)
                        chunk = {
                            "id": "chatcmpl-proxy",
                            "object": "chat.completion.chunk",
                            "created": created,
                            "model": requested_model,
                            "system_fingerprint": None,
                            "choices": [
                                {
                                    "index": 0,
                                    "delta": {"content": payload},
                                    "logprobs": None,
                                    "finish_reason": None,
                                }
                            ],
                        }
                        yield f"data: {json.dumps(chunk)}\n\n"
                    elif kind == "tool_calls":
                        tool_calls = payload
                        chunk = {
                            "id": "chatcmpl-proxy",
                            "object": "chat.completion.chunk",
                            "created": created,
                            "model": requested_model,
                            "system_fingerprint": None,
                            "choices": [
                                {
                                    "index": 0,
                                    "delta": {
                                        "tool_calls": [
                                            {"index": i, **tc}
                                            for i, tc in enumerate(tool_calls)
                                        ]
                                    },
                                    "logprobs": None,
                                    "finish_reason": None,
                                }
                            ],
                        }
                        yield f"data: {json.dumps(chunk)}\n\n"
                    elif kind == "retry":
                        # SSE comment line — valid per spec, ignored by every
                        # standard client parser (including OpenCode/Copilot's
                        # OpenAI SSE handling). Keeps the socket alive during
                        # backoff so the client never sees a timeout or a
                        # dropped stream while we wait out a 429/5xx upstream.
                        yield f": retrying upstream request ({payload:.1f}s backoff)\n\n"
                    else:
                        input_tokens, output_tokens = payload
            except Exception as e:
                err = str(e)[:500]
                print(f"❌  Vertex error: {err}", flush=True)
                import traceback

                traceback.print_exc()
                yield f"data: {json.dumps({'id': 'chatcmpl-proxy', 'object': 'chat.completion.chunk', 'created': created, 'model': requested_model, 'system_fingerprint': None, 'choices': [{'index': 0, 'delta': {'content': f'[proxy error: {err}]'}, 'logprobs': None, 'finish_reason': 'stop'}]})}\n\n"
                yield "data: [DONE]\n\n"
                return

            log_cost(
                input_tokens,
                output_tokens,
                route,
                requested_model,
                caller,
                prompt_text,
                "".join(full_text),
            )

            finish_reason = (
                "tool_calls" if tool_calls else ("stop" if full_text else "length")
            )
            yield f"data: {json.dumps({'id': 'chatcmpl-proxy', 'object': 'chat.completion.chunk', 'created': created, 'model': requested_model, 'system_fingerprint': None, 'choices': [{'index': 0, 'delta': {}, 'logprobs': None, 'finish_reason': finish_reason}]})}\n\n"
            yield "data: [DONE]\n\n"

        return StreamingResponse(generate(), media_type="text/event-stream")

    try:
        text, tool_calls, input_tokens, output_tokens = generate_once(
            route,
            system_instruction,
            raw_messages,
            max_tokens,
            temperature,
            tools=tools,
            tool_choice=tool_choice,
        )
    except Exception as e:
        err = str(e)[:500]
        print(f"❌  Vertex error: {err}", flush=True)
        import traceback

        traceback.print_exc()
        return Response(
            content=json.dumps({"error": {"type": "api_error", "message": err}}),
            status_code=502,
            media_type="application/json",
        )

    log_cost(
        input_tokens, output_tokens, route, requested_model, caller, prompt_text, text
    )
    message = {"role": "assistant", "content": text or None}
    if tool_calls:
        message["tool_calls"] = tool_calls
    return {
        "id": "chatcmpl-proxy",
        "object": "chat.completion",
        "created": int(datetime.datetime.now().timestamp()),
        "model": requested_model,
        "system_fingerprint": None,
        "choices": [
            {
                "index": 0,
                "message": message,
                "logprobs": None,
                "finish_reason": "tool_calls" if tool_calls else "stop",
            }
        ],
        "usage": {
            "prompt_tokens": input_tokens,
            "completion_tokens": output_tokens,
            "total_tokens": input_tokens + output_tokens,
        },
    }


# ── Routes: OpenAI model listing (opencode/Copilot query this) ─────────────────


@app.get("/v1/models")
async def list_models(caller: str = Depends(require_api_key)):
    now = int(datetime.datetime.now().timestamp())
    return {
        "object": "list",
        "data": [
            {
                "id": alias,
                "object": "model",
                "created": now,
                "owned_by": "vertex-ai",
                "name": info.get("label", alias),
                "display_name": info.get("label", alias),
                "description": (
                    f"{info.get('label', alias)} — "
                    f"${info['input_cost'] * 1_000_000:.2f}/$"
                    f"{info['output_cost'] * 1_000_000:.2f} per 1M tokens"
                ),
            }
            for alias, info in MODEL_ROUTES.items()
        ],
    }


@app.get("/health")
async def health():
    return {
        "status": "ok",
        "models": list(MODEL_ROUTES.keys()),
        "target": f"vertex-ai://{GCP_PROJECT}",
        "db": _DB_OK,
        "csv_logging": True,
        "session": session_stats,
    }


# ── Route: Web dashboard ───────────────────────────────────────────────────────


@app.get("/", response_class=HTMLResponse)
async def dashboard(caller: str = Depends(require_api_key)):
    logs = fetch_recent_logs(50)
    rows_html = ""
    if logs:
        for i, r in enumerate(logs):
            ts = r["ts"].strftime("%Y-%m-%d %H:%M:%S") if r.get("ts") else ""
            model = r.get("model_name") or ""
            client_name = r.get("client") or ""
            intok = r.get("input_tokens") or 0
            outtok = r.get("output_tokens") or 0
            cost = float(r.get("cost_usd") or 0)
            prompt = (r.get("prompt") or "").replace("<", "&lt;").replace(">", "&gt;")
            completion = (
                (r.get("completion") or "").replace("<", "&lt;").replace(">", "&gt;")
            )
            rows_html += f"""
            <tr class="border-b border-slate-700 hover:bg-slate-800/50">
              <td class="px-4 py-2 text-slate-400 whitespace-nowrap">{ts}</td>
              <td class="px-4 py-2"><span class="px-2 py-1 rounded bg-slate-700 text-xs">{model}</span></td>
              <td class="px-4 py-2 text-slate-400">{client_name}</td>
              <td class="px-4 py-2 text-right">{intok:,}</td>
              <td class="px-4 py-2 text-right">{outtok:,}</td>
              <td class="px-4 py-2 text-right text-emerald-400">${cost:.6f}</td>
              <td class="px-4 py-2 text-center">
                <button onclick="toggle('d{i}')" class="text-sky-400 hover:underline text-xs">view</button>
              </td>
            </tr>
            <tr id="d{i}" class="hidden bg-slate-900/80">
              <td colspan="7" class="px-4 py-3">
                <div class="text-xs font-semibold text-sky-300 mb-1">PROMPT</div>
                <pre class="whitespace-pre-wrap text-xs text-slate-300 bg-slate-950 p-3 rounded mb-3 max-h-60 overflow-auto">{prompt}</pre>
                <div class="text-xs font-semibold text-emerald-300 mb-1">COMPLETION</div>
                <pre class="whitespace-pre-wrap text-xs text-slate-300 bg-slate-950 p-3 rounded max-h-60 overflow-auto">{completion}</pre>
              </td>
            </tr>"""
    else:
        note = (
            "No database logs yet."
            if _DB_OK
            else "PostgreSQL not connected — showing session stats only."
        )
        rows_html = f'<tr><td colspan="7" class="px-4 py-6 text-center text-slate-500">{note}</td></tr>'

    s = session_stats
    html = f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Vertex AI Proxy — Dashboard</title>
  <script src="https://cdn.tailwindcss.com"></script>
  <meta http-equiv="refresh" content="15">
</head>
<body class="bg-slate-950 text-slate-100 font-sans">
  <div class="max-w-7xl mx-auto p-6">
    <div class="flex items-center justify-between mb-6">
      <div>
        <h1 class="text-2xl font-bold">🚀 Vertex AI Proxy</h1>
        <p class="text-slate-400 text-sm">Claude Code + opencode + Copilot → GCP Vertex AI {'· DB connected' if _DB_OK else '· DB offline'} · CSV logging active</p>
      </div>
      <div class="text-xs text-slate-500">auto-refresh 15s</div>
    </div>

    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
      <div class="bg-gradient-to-br from-emerald-900/40 to-slate-900 border border-emerald-700/40 rounded-xl p-5">
        <div class="text-emerald-400 text-xs uppercase tracking-wide">Session Spend</div>
        <div class="text-3xl font-bold mt-1">${s['total_cost_usd']:.6f}</div>
      </div>
      <div class="bg-gradient-to-br from-sky-900/40 to-slate-900 border border-sky-700/40 rounded-xl p-5">
        <div class="text-sky-400 text-xs uppercase tracking-wide">Requests</div>
        <div class="text-3xl font-bold mt-1">{s['requests']:,}</div>
      </div>
      <div class="bg-gradient-to-br from-indigo-900/40 to-slate-900 border border-indigo-700/40 rounded-xl p-5">
        <div class="text-indigo-400 text-xs uppercase tracking-wide">Input Tokens</div>
        <div class="text-3xl font-bold mt-1">{s['input_tokens']:,}</div>
      </div>
      <div class="bg-gradient-to-br from-fuchsia-900/40 to-slate-900 border border-fuchsia-700/40 rounded-xl p-5">
        <div class="text-fuchsia-400 text-xs uppercase tracking-wide">Output Tokens</div>
        <div class="text-3xl font-bold mt-1">{s['output_tokens']:,}</div>
      </div>
    </div>

    <div class="bg-slate-900 border border-slate-800 rounded-xl overflow-hidden">
      <div class="px-4 py-3 border-b border-slate-800 font-semibold">Last 50 Requests</div>
      <div class="overflow-x-auto">
        <table class="w-full text-sm">
          <thead class="bg-slate-800/60 text-slate-400 text-xs uppercase">
            <tr>
              <th class="px-4 py-2 text-left">Timestamp</th>
              <th class="px-4 py-2 text-left">Model</th>
              <th class="px-4 py-2 text-left">Client</th>
              <th class="px-4 py-2 text-right">In</th>
              <th class="px-4 py-2 text-right">Out</th>
              <th class="px-4 py-2 text-right">Cost</th>
              <th class="px-4 py-2 text-center">Raw</th>
            </tr>
          </thead>
          <tbody>{rows_html}</tbody>
        </table>
      </div>
    </div>
  </div>

  <script>
    function toggle(id) {{
      const el = document.getElementById(id);
      if (el) el.classList.toggle('hidden');
    }}
  </script>
</body>
</html>"""
    return HTMLResponse(content=html)


# ── Entry point ────────────────────────────────────────────────────────────────

if __name__ == "__main__":
    port = int(os.getenv("PORT", "4000"))
    init_db()
    print("✅  CSV logging enabled → request_logs.csv")
    print(f"🚀  Proxy listening on http://localhost:{port}")
    print(f"📊  Dashboard  → http://localhost:{port}/")
    print(
        f"🔒  Auth       : {'ENABLED (' + str(len(PROXY_API_KEYS)) + ' key(s))' if PROXY_API_KEYS else 'disabled — set PROXY_API_KEYS before exposing this beyond localhost'}\n"
    )
    uvicorn.run(app, host="0.0.0.0", port=port, log_level="warning")
