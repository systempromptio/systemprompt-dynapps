"""Microsoft Entra ID and Microsoft Graph, simulated for two companies.

`--certs DIR` writes a throwaway CA and a localhost server certificate into
DIR and exits; the caller trusts DIR/ca.pem (SSL_CERT_FILE) before it starts.
`--serve DIR` serves, on loopback only:

* Entra over TLS with DIR's certificate: GET /{tenant}/oauth2/v2.0/authorize
  signs in the account named by `login_hint` and redirects back with a code
  bound to the PKCE challenge; POST /{tenant}/oauth2/v2.0/token redeems it
  (S256 verifier, client id and secret, no `resource`, `offline_access` and
  `openid` in the scope) for a Graph token and an unsigned ID token whose
  `tid` is the account's own tenant.
* Graph over plain HTTP under /v1.0: each access token belongs to one account
  in one tenant, and every mailbox, attachment and drive item lives in that
  tenant, so a token of tenant A never reaches tenant B's data. GET /_sim/log
  lists every Graph request as (method, path, tenant, account).

It prints one JSON line with both origins. No network beyond loopback, no
real accounts.
"""
import base64
import hashlib
import http.server
import json
import pathlib
import re
import ssl
import subprocess
import sys
import threading
import time
import urllib.parse

CLIENT = "m365-sim-client"
SECRET = "m365-sim-secret"
TENANT_A = "11111111-1111-4111-8111-111111111111"
TENANT_B = "22222222-2222-4222-8222-222222222222"
TENANT_C = "33333333-3333-4333-8333-333333333333"
ACCOUNTS = {
    "ana@empresa-a.example": (TENANT_A, "0000aaaa-0000-4000-8000-00000000aaaa"),
    "bruno@empresa-b.example": (TENANT_B, "0000bbbb-0000-4000-8000-00000000bbbb"),
    "carla@empresa-c.example": (TENANT_C, "0000cccc-0000-4000-8000-00000000cccc"),
}
INVOICE_A = b"%PDF-1.4 factura de la empresa A\n"
MAILBOXES = {
    TENANT_A: {"msg-a-1": {"id": "msg-a-1", "subject": "Pedido de la empresa A", "isRead": False,
                           "from": {"emailAddress": {"address": "cliente@empresa-a.example"}},
                           "attachments": {"att-a-1": {"@odata.type": "#microsoft.graph.fileAttachment",
                                                       "id": "att-a-1", "name": "factura-a.pdf",
                                                       "contentType": "application/pdf",
                                                       "contentBytes": base64.b64encode(INVOICE_A).decode()}}}},
    TENANT_B: {"msg-b-1": {"id": "msg-b-1", "subject": "Pedido de la empresa B", "isRead": False,
                           "from": {"emailAddress": {"address": "cliente@empresa-b.example"}},
                           "attachments": {}}},
    TENANT_C: {},
}
DRIVES = {TENANT_A: {}, TENANT_B: {}, TENANT_C: {}}
TOKEN_PATH = re.compile(r"^/(organizations|[0-9a-f-]{36})/oauth2/v2\.0/(authorize|token)$")
lock = threading.Lock()
codes = {}
refreshes = {}
tokens = {}
log = []
counter = [0]


def openssl(root, *args):
    subprocess.run(["openssl", *args], cwd=root, check=True,
                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def write_certs(root):
    root.mkdir(parents=True, exist_ok=True)
    openssl(root, "req", "-x509", "-newkey", "rsa:2048", "-nodes", "-keyout", "ca.key", "-out", "ca.pem",
            "-days", "1", "-subj", "/CN=M365 simulation CA")
    openssl(root, "req", "-newkey", "rsa:2048", "-nodes", "-keyout", "server.key", "-out", "server.csr",
            "-subj", "/CN=localhost")
    (root / "extensions").write_text("subjectAltName=DNS:localhost,IP:127.0.0.1\nbasicConstraints=CA:FALSE\n"
                                     "keyUsage=digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\n")
    openssl(root, "x509", "-req", "-in", "server.csr", "-CA", "ca.pem", "-CAkey", "ca.key", "-CAcreateserial",
            "-out", "server.pem", "-days", "1", "-extfile", "extensions")


def nonce(prefix):
    with lock:
        counter[0] += 1
        return f"{prefix}-{counter[0]}"


def b64(data):
    return base64.urlsafe_b64encode(data).rstrip(b"=").decode()


def id_token(origin, account):
    tid, oid = ACCOUNTS[account]
    claims = {"iss": f"{origin}/{tid}/v2.0", "aud": CLIENT, "tid": tid, "oid": oid, "sub": oid,
              "preferred_username": account, "exp": int(time.time()) + 3600}
    return ".".join([b64(json.dumps({"alg": "RS256", "typ": "JWT"}).encode()),
                     b64(json.dumps(claims).encode()), b64(b"simulated-signature")])


class Base(http.server.BaseHTTPRequestHandler):
    def log_message(self, *args):
        pass

    def reply(self, status, body=None, content_type="application/json", headers=()):
        data = b"" if body is None else (body if isinstance(body, bytes) else json.dumps(body).encode())
        self.send_response(status)
        for name, value in headers:
            self.send_header(name, value)
        if body is not None:
            self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def body(self):
        return self.rfile.read(int(self.headers.get("Content-Length", 0)))


class Entra(Base):
    def do_GET(self):
        url = urllib.parse.urlsplit(self.path)
        match = TOKEN_PATH.match(url.path)
        if not match or match.group(2) != "authorize":
            return self.reply(404, {})
        query = dict(urllib.parse.parse_qsl(url.query))
        account = query.get("login_hint")
        if (query.get("client_id") != CLIENT or query.get("response_type") != "code"
                or query.get("code_challenge_method") != "S256" or account not in ACCOUNTS):
            return self.reply(400, {"error": "invalid_request"})
        code = nonce("code")
        codes[code] = (query["code_challenge"], query["redirect_uri"], account)
        target = query["redirect_uri"] + "?" + urllib.parse.urlencode({"code": code, "state": query["state"]})
        return self.reply(302, None, headers=[("Location", target)])

    def do_POST(self):
        match = TOKEN_PATH.match(self.path)
        if not match or match.group(2) != "token":
            return self.reply(404, {})
        fields = {k: v[0] for k, v in urllib.parse.parse_qs(self.body().decode()).items()}
        if "resource" in fields:
            return self.reply(400, {"error": "invalid_request", "error_description": "AADSTS901002"})
        if fields.get("client_id") != CLIENT or fields.get("client_secret") != SECRET:
            return self.reply(401, {"error": "invalid_client"})
        scopes = fields.get("scope", "").split()
        if "offline_access" not in scopes or "openid" not in scopes:
            return self.reply(400, {"error": "invalid_scope"})
        if fields.get("grant_type") == "authorization_code":
            issued = codes.pop(fields.get("code"), None)
            if issued is None:
                return self.reply(400, {"error": "invalid_grant"})
            challenge, redirect, account = issued
            verifier = fields.get("code_verifier", "")
            if b64(hashlib.sha256(verifier.encode()).digest()) != challenge or fields.get("redirect_uri") != redirect:
                return self.reply(400, {"error": "invalid_grant"})
        elif fields.get("grant_type") == "refresh_token":
            account = refreshes.pop(fields.get("refresh_token"), None)
            if account is None:
                return self.reply(400, {"error": "invalid_grant"})
        else:
            return self.reply(400, {"error": "unsupported_grant_type"})
        access, refresh = nonce("graph-" + ACCOUNTS[account][0][:8]), nonce("refresh")
        tokens[access] = account
        refreshes[refresh] = account
        origin = "https://localhost:" + str(self.server.server_port)
        return self.reply(200, {"access_token": access, "token_type": "Bearer", "expires_in": 3600,
                                "scope": " ".join(scopes), "refresh_token": refresh,
                                "id_token": id_token(origin, account)})


def graph_error(code, status):
    return status, {"error": {"code": code, "message": code}}


def selected(resource, select):
    if not select:
        return resource
    fields = set(select.split(",")) | {"@odata.type"}
    return {k: v for k, v in resource.items() if k in fields}


def mail_route(method, parts, mailbox, payload, select=None):
    if parts == ["messages"] and method == "POST":
        message_id = nonce("draft")
        mailbox[message_id] = {"id": message_id, "subject": payload.get("subject", ""), "isRead": True,
                               "isDraft": True, "attachments": {}}
        return 201, {k: v for k, v in mailbox[message_id].items() if k != "attachments"}
    if parts[-1:] == ["messages"]:
        return 200, {"value": [{k: v for k, v in m.items() if k != "attachments"} for m in mailbox.values()]}
    message = mailbox.get(parts[1]) if len(parts) > 1 else None
    if message is None:
        return graph_error("ErrorItemNotFound", 404)
    rest = parts[2:]
    if not rest and method == "GET":
        return 200, {k: v for k, v in message.items() if k != "attachments"}
    if not rest and method == "PATCH":
        message.update({k: v for k, v in payload.items() if k in ("isRead", "importance", "categories")})
        return 200, {k: v for k, v in message.items() if k != "attachments"}
    if rest == ["send"] and method == "POST":
        message["isDraft"] = False
        return 202, None
    if rest == ["attachments"] and method == "POST":
        attachment_id = nonce("att")
        message["attachments"][attachment_id] = dict(payload, id=attachment_id)
        return 201, {"id": attachment_id, "name": payload.get("name"),
                     "size": len(base64.b64decode(payload.get("contentBytes", "")))}
    if rest == ["attachments"] and method == "GET":
        return 200, {"value": [{k: v for k, v in a.items() if k != "contentBytes"}
                               for a in message["attachments"].values()]}
    if len(rest) == 2 and rest[0] == "attachments" and method == "GET":
        attachment = message["attachments"].get(rest[1])
        return (200, selected(attachment, select)) if attachment else graph_error("ErrorItemNotFound", 404)
    if len(rest) == 3 and rest[0] == "attachments" and rest[2] == "$value" and method == "GET":
        attachment = message["attachments"].get(rest[1])
        if attachment is None or "contentBytes" not in attachment:
            return graph_error("ErrorItemNotFound", 404)
        return 200, base64.b64decode(attachment["contentBytes"]), attachment.get("contentType") or "application/octet-stream"
    return graph_error("BadRequest", 400)


class Graph(Base):
    def handle_any(self, method):
        url = urllib.parse.urlsplit(self.path)
        if url.path == "/_sim/log" and method == "GET":
            return self.reply(200, log)
        if url.path == "/_sim/state" and method == "GET":
            return self.reply(200, {t: {m: {"attachments": sorted(v["attachments"]), "isDraft": v.get("isDraft", False),
                                            "isRead": v.get("isRead")} for m, v in box.items()}
                                    for t, box in MAILBOXES.items()} | {"drives": DRIVES_VIEW()})
        account = tokens.get(self.headers.get("Authorization", "").removeprefix("Bearer "))
        if account is None:
            return self.reply(401, {"error": {"code": "InvalidAuthenticationToken", "message": "unknown token"}})
        tenant = ACCOUNTS[account][0]
        path = urllib.parse.unquote(url.path)
        log.append({"method": method, "path": path, "tenant": tenant, "account": account})
        raw = self.body() if method in ("POST", "PATCH", "PUT") else b""
        if not path.startswith("/v1.0/me"):
            return self.reply(*graph_error("ResourceNotFound", 404))
        rest = path[len("/v1.0/me"):]
        if rest == "" and method == "GET":
            return self.reply(200, {"id": ACCOUNTS[account][1], "userPrincipalName": account, "mail": account})
        if method == "PUT" and rest.startswith("/drive/root:/") and rest.endswith(":/content"):
            name = rest[len("/drive/root:/"):-len(":/content")]
            item_id = nonce("item")
            DRIVES[tenant][item_id] = {"name": name, "size": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}
            return self.reply(201, {"id": item_id, "name": name.rsplit("/", 1)[-1], "size": len(raw)})
        parts = [p for p in rest.split("/") if p]
        if len(parts) > 2 and parts[0] == "mailFolders":
            parts = parts[2:]
        if parts and parts[0] == "messages":
            payload = json.loads(raw or b"{}")
            select = dict(urllib.parse.parse_qsl(url.query)).get("$select")
            return self.reply(*mail_route(method, parts, MAILBOXES[tenant], payload, select))
        return self.reply(*graph_error("ResourceNotFound", 404))

    def do_GET(self):
        self.handle_any("GET")

    def do_POST(self):
        self.handle_any("POST")

    def do_PATCH(self):
        self.handle_any("PATCH")

    def do_PUT(self):
        self.handle_any("PUT")

    def do_DELETE(self):
        self.handle_any("DELETE")


def DRIVES_VIEW():
    return {tenant: list(items.values()) for tenant, items in DRIVES.items()}


def serve(root):
    entra = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Entra)
    context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    context.load_cert_chain(root / "server.pem", root / "server.key")
    entra.socket = context.wrap_socket(entra.socket, server_side=True)
    graph = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Graph)
    threading.Thread(target=graph.serve_forever, daemon=True).start()
    print(json.dumps({"entra": f"https://localhost:{entra.server_port}",
                      "graph": f"http://127.0.0.1:{graph.server_port}/v1.0",
                      "client_id": CLIENT, "client_secret": SECRET,
                      "tenants": {"a": TENANT_A, "b": TENANT_B, "c": TENANT_C}}), flush=True)
    entra.serve_forever()


if __name__ == "__main__":
    mode, directory = sys.argv[1], pathlib.Path(sys.argv[2])
    if mode == "--certs":
        write_certs(directory)
    else:
        serve(directory)
