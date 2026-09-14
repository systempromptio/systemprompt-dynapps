# syntax=docker/dockerfile:1.7
FROM ghcr.io/astral-sh/uv:0.11.6-python3.13-trixie@sha256:b3c543b6c4f23a5f2df22866bd7857e5d304b67a564f4feab6ac22044dde719b AS uv_source
FROM node:22-trixie-slim
ARG CLAUDE_VERSION=2.1.270
ARG OPENCODE_VERSION=1.18.29
ARG CODEX_VERSION=0.154.0
ARG HERMES_COMMIT=345cd2b057a452236de401d3534b8502a7465e8d
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates git ripgrep python3 python3-venv bubblewrap \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 1001 tester \
    && npm install --global @anthropic-ai/claude-code@${CLAUDE_VERSION} opencode-ai@${OPENCODE_VERSION} @openai/codex@${CODEX_VERSION} \
    && npm cache clean --force \
    && install -d -o tester -g tester /home/tester/work
RUN node -e 'const fs=require("fs"),path=require("path"),{createRequire}=require("module"); const root="/usr/local/lib/node_modules/@openai/codex/package.json"; const req=createRequire(root); const arch=process.arch==="x64"?"x86_64":process.arch==="arm64"?"aarch64":null; if(!arch)throw Error("unsupported Codex architecture"); const pkg=req.resolve("@openai/codex-linux-"+process.arch+"/package.json"); fs.cpSync(path.join(path.dirname(pkg),"vendor",arch+"-unknown-linux-musl"),"/opt/systemprompt/codex",{recursive:true});'
COPY --from=uv_source /usr/local/bin/uv /usr/local/bin/uv
RUN git init /opt/hermes-source \
    && git -C /opt/hermes-source fetch --depth=1 https://github.com/NousResearch/hermes-agent.git ${HERMES_COMMIT} \
    && git -C /opt/hermes-source checkout --detach FETCH_HEAD \
    && test "$(git -C /opt/hermes-source rev-parse HEAD)" = "${HERMES_COMMIT}" \
    && rm -rf /opt/hermes-source/.git \
    && cd /opt/hermes-source \
    && UV_PYTHON=/usr/bin/python3 uv sync --frozen --no-dev --no-install-project --extra mcp \
    && uv pip install --no-cache-dir --no-deps -e .
COPY codex-runner.cjs /opt/systemprompt/codex-runner.cjs
COPY hermes-runner.cjs /opt/systemprompt/hermes-runner.cjs
ENV HOME=/home/tester \
    DISABLE_AUTOUPDATER=1 \
    CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1 \
    OPENCODE_DISABLE_AUTOUPDATE=1 \
    XDG_CONFIG_HOME=/home/tester/.config \
    XDG_CACHE_HOME=/home/tester/.cache \
    XDG_DATA_HOME=/home/tester/.local/share
USER tester
WORKDIR /home/tester/work
