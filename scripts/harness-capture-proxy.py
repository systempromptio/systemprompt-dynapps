#!/usr/bin/env python3
"""Recording reverse proxy for `just smoke-harness-local`.

Sits between a real coding client and the local gateway, forwards every
request unchanged (streaming the reply back as it arrives), and writes each
inference request body plus its non-credential headers to CAPTURE_DIR as one
JSON file. Those captures become the sanitised fixtures in
tests/fixtures/harness/.

    harness-capture-proxy.py <listen-port> <upstream-base-url> <capture-dir> <client>
"""
import http.server
import json
import os
import sys
import threading
import urllib.error
import urllib.request

PORT, UPSTREAM, CAPTURE_DIR, CLIENT = int(sys.argv[1]), sys.argv[2].rstrip("/"), sys.argv[3], sys.argv[4]
CREDENTIALS = {"authorization", "x-api-key", "cookie", "proxy-authorization"}
INFERENCE = ("/v1/messages", "/v1/chat/completions", "/v1/responses")
HOP = {"connection", "keep-alive", "transfer-encoding", "content-length", "host", "accept-encoding"}
counter = {"n": 0}
lock = threading.Lock()


class Proxy(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, *args):
        pass

    def _forward(self):
        length = int(self.headers.get("content-length") or 0)
        body = self.rfile.read(length) if length else None
        path = self.path.split("?")[0]
        if body and path.endswith(INFERENCE):
            with lock:
                counter["n"] += 1
                n = counter["n"]
            headers = {k.lower(): v for k, v in self.headers.items() if k.lower() not in CREDENTIALS | HOP}
            try:
                parsed = json.loads(body)
            except ValueError:
                parsed = None
            with open(os.path.join(CAPTURE_DIR, f"{CLIENT}-{n:03d}.json"), "w") as f:
                json.dump({"client": CLIENT, "path": path, "headers": headers, "body": parsed}, f, indent=1)
        req = urllib.request.Request(UPSTREAM + self.path, data=body, method=self.command)
        for k, v in self.headers.items():
            if k.lower() not in HOP:
                req.add_header(k, v)
        try:
            resp = urllib.request.urlopen(req, timeout=600)
        except urllib.error.HTTPError as e:
            resp = e
        self.send_response(resp.status if hasattr(resp, "status") else resp.code)
        for k, v in resp.headers.items():
            if k.lower() not in HOP:
                self.send_header(k, v)
        self.send_header("transfer-encoding", "chunked")
        self.end_headers()
        while True:
            chunk = resp.read1(65536) if hasattr(resp, "read1") else resp.read(65536)
            if not chunk:
                break
            self.wfile.write(b"%x\r\n%s\r\n" % (len(chunk), chunk))
            self.wfile.flush()
        self.wfile.write(b"0\r\n\r\n")

    do_GET = do_POST = do_PUT = do_DELETE = do_PATCH = _forward


os.makedirs(CAPTURE_DIR, exist_ok=True)
http.server.ThreadingHTTPServer(("127.0.0.1", PORT), Proxy).serve_forever()
