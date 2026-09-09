# syntax=docker/dockerfile:1.7
# Build with deploy/clean-client as context. No host client state is copied.
FROM node:22-trixie-slim
ARG CLAUDE_VERSION=2.1.266
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl git jq less ripgrep python3 dbus libdbus-1-3 sudo procps \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --shell /bin/bash --uid 1001 tester \
    && echo 'tester ALL=(ALL) NOPASSWD:ALL' > /etc/sudoers.d/tester \
    && chmod 0440 /etc/sudoers.d/tester \
    && install -d -o tester -g tester /opt/npm-global /home/tester/work
ENV NPM_CONFIG_PREFIX=/opt/npm-global PATH=/opt/npm-global/bin:$PATH
RUN printf 'export PATH=/opt/npm-global/bin:$HOME/.local/bin:$PATH\n' > /etc/profile.d/india-path.sh
USER tester
RUN npm install -g @anthropic-ai/claude-code@${CLAUDE_VERSION} && npm cache clean --force
ENV DISABLE_AUTOUPDATER=1 CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1
WORKDIR /home/tester/work
ENTRYPOINT ["sleep", "infinity"]
