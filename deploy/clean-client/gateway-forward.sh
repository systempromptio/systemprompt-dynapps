#!/usr/bin/env bash
# Loopback TCP forwarder for the clean client's feedback plane.
#
#     gateway-forward <port> [upstream-host]
#
# Listens on 127.0.0.1:<port> inside the container and forwards every byte to
# <upstream-host>:<port> (default host.docker.internal). The bridge is then
# pointed at http://localhost:<port> instead of the Docker alias.
#
# Why: on `bridge login` the bridge self-enrols a device credential — a bearer
# token — and it refuses to do that over plain http unless the host is loopback.
# `http://host.docker.internal:*` is neither https nor loopback, so a container
# that talks to the gateway by that name signs in fine and never gets a device,
# and nothing it reports afterwards is attributed. Forwarding through 127.0.0.1
# keeps the bridge's rule intact and still reaches the host's gateway.
#
# Idempotent: if the port already answers, this exits 0 without starting a
# second copy. The forwarder runs detached, logs to ~/.cache, and this script
# returns only once the port accepts connections (or fails after ~10s).
#
# Node is used because the image already carries it for Claude Code and
# OpenCode; nothing else is installed for this.
set -euo pipefail

PORT="${1:?usage: gateway-forward <port> [upstream-host]}"
UPSTREAM="${2:-host.docker.internal}"

case "$PORT" in
    ''|*[!0-9]*) echo "gateway-forward: port must be numeric, got '$PORT'" >&2; exit 2 ;;
esac

command -v node >/dev/null 2>&1 || { echo "gateway-forward: node is not installed" >&2; exit 1; }

CACHE_DIR="${XDG_CACHE_HOME:-$HOME/.cache}"
LOG="$CACHE_DIR/gateway-forward.$PORT.log"
PIDFILE="$CACHE_DIR/gateway-forward.$PORT.pid"
mkdir -p "$CACHE_DIR"

# A connect probe with a short timeout; exit 0 only when the port accepts.
port_answers() {
    FWD_PORT="$PORT" node -e '
const net = require("net");
const s = net.connect({ host: "127.0.0.1", port: Number(process.env.FWD_PORT) });
s.setTimeout(1000);
s.on("connect", () => { s.destroy(); process.exit(0); });
s.on("timeout", () => { s.destroy(); process.exit(1); });
s.on("error", () => process.exit(1));
' >/dev/null 2>&1
}

if port_answers; then
    if [ -f "$PIDFILE" ] && kill -0 "$(cat "$PIDFILE")" 2>/dev/null; then
        echo "    forwarder already running (pid $(cat "$PIDFILE")) on 127.0.0.1:$PORT -> $UPSTREAM:$PORT"
    else
        echo "    127.0.0.1:$PORT already answers — not starting a forwarder"
    fi
    exit 0
fi

# Two independent sockets per connection, torn down together: an error or
# close on either side destroys both, so a half-open pair never lingers.
FORWARDER='
const net = require("net");
const port = Number(process.env.FWD_PORT);
const upstreamHost = process.env.FWD_UPSTREAM;
const log = (msg) => console.log(new Date().toISOString() + " " + msg);
const server = net.createServer((client) => {
  const upstream = net.connect({ host: upstreamHost, port });
  const teardown = (why) => (err) => {
    if (err) log(why + ": " + err.message);
    client.destroy();
    upstream.destroy();
  };
  client.on("error", teardown("client"));
  upstream.on("error", teardown("upstream"));
  client.on("close", () => upstream.destroy());
  upstream.on("close", () => client.destroy());
  client.pipe(upstream);
  upstream.pipe(client);
});
server.on("error", (err) => {
  log("listen failed: " + err.message);
  process.exit(err.code === "EADDRINUSE" ? 3 : 1);
});
server.listen(port, "127.0.0.1", () =>
  log("listening on 127.0.0.1:" + port + " -> " + upstreamHost + ":" + port));
'

# setsid + nohup so the forwarder outlives the shell that started it — the
# bootstrap ends with `exec bash -l`, and a child tied to the old job table
# would take the forwarder down with it.
FWD_PORT="$PORT" FWD_UPSTREAM="$UPSTREAM" \
    setsid nohup node -e "$FORWARDER" >>"$LOG" 2>&1 < /dev/null &
echo $! > "$PIDFILE"

for _ in $(seq 1 40); do
    if port_answers; then
        echo "    forwarding 127.0.0.1:$PORT -> $UPSTREAM:$PORT (pid $(cat "$PIDFILE"), log $LOG)"
        exit 0
    fi
    if ! kill -0 "$(cat "$PIDFILE")" 2>/dev/null; then
        echo "gateway-forward: the forwarder exited before listening — $LOG says:" >&2
        tail -n 5 "$LOG" >&2 || true
        exit 1
    fi
    sleep 0.25
done

echo "gateway-forward: 127.0.0.1:$PORT did not start answering within 10s — see $LOG" >&2
exit 1
