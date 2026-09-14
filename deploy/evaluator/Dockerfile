# syntax=docker/dockerfile:1.7
FROM node:22-trixie-slim
ARG CLAUDE_VERSION=2.1.270
ARG OPENCODE_VERSION=1.18.29
ARG CODEX_VERSION=0.154.0
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates git ripgrep \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 1001 tester \
    && npm install --global @anthropic-ai/claude-code@${CLAUDE_VERSION} opencode-ai@${OPENCODE_VERSION} @openai/codex@${CODEX_VERSION} \
    && npm cache clean --force \
    && install -d -o tester -g tester /home/tester/work
RUN node -e 'const fs=require("fs"),path=require("path"),{createRequire}=require("module"); const root="/usr/local/lib/node_modules/@openai/codex/package.json"; const req=createRequire(root); const arch=process.arch==="x64"?"x86_64":process.arch==="arm64"?"aarch64":null; if(!arch)throw Error("unsupported Codex architecture"); const pkg=req.resolve("@openai/codex-linux-"+process.arch+"/package.json"); fs.cpSync(path.join(path.dirname(pkg),"vendor",arch+"-unknown-linux-musl"),"/opt/systemprompt/codex",{recursive:true});'
COPY codex-runner.cjs /opt/systemprompt/codex-runner.cjs
ENV HOME=/home/tester \
    DISABLE_AUTOUPDATER=1 \
    CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1 \
    OPENCODE_DISABLE_AUTOUPDATE=1 \
    XDG_CONFIG_HOME=/home/tester/.config \
    XDG_CACHE_HOME=/home/tester/.cache \
    XDG_DATA_HOME=/home/tester/.local/share
USER tester
WORKDIR /home/tester/work
