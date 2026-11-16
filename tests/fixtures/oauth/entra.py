"""Loopback TLS stand-in for the Microsoft Entra ID v2.0 token endpoint.

Answers POST /{organizations|tenant-id}/oauth2/v2.0/token the way Entra does:
a Bearer access token, a rotating refresh token and an unsigned-for-test ID
token whose claims depend on the authorization code or refresh token, so one
server covers the admitted tenant, a foreign tenant, a forged issuer and a
missing ID token on the exchange or on a refresh. A request carrying
`resource`, a wrong client or secret, a scope without `offline_access`/`openid` or a bad PKCE verifier is refused, as
Entra refuses it. No external accounts or network services are involved.
"""
import base64
import http.server
import json
import pathlib
import re
import ssl
import subprocess
import sys
import time
import urllib.parse

root = pathlib.Path(sys.argv[1])


def openssl(*args):
    subprocess.run(["openssl", *args], cwd=root, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


openssl("req", "-x509", "-newkey", "rsa:2048", "-nodes", "-keyout", "ca.key", "-out", "ca.pem", "-days", "1", "-subj", "/CN=Entra test CA")
openssl("req", "-newkey", "rsa:2048", "-nodes", "-keyout", "server.key", "-out", "server.csr", "-subj", "/CN=localhost")
(root / "extensions").write_text("subjectAltName=DNS:localhost,IP:127.0.0.1\nbasicConstraints=CA:FALSE\nkeyUsage=digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\n")
openssl("x509", "-req", "-in", "server.csr", "-CA", "ca.pem", "-CAkey", "ca.key", "-CAcreateserial", "-out", "server.pem", "-days", "1", "-extfile", "extensions")

CLIENT = "entra-client"
SECRET = "entra-secret"
TENANT_A = "11111111-1111-4111-8111-111111111111"
TENANT_B = "22222222-2222-4222-8222-222222222222"
OID = "0000aaaa-0000-4000-8000-00000000aaaa"
TOKEN_PATH = re.compile(r"^/(organizations|[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})/oauth2/v2\.0/token$")

# code or refresh token -> (tid claim, tenant named in iss, next refresh token)
CODES = {
    "code-allowed": (TENANT_A, TENANT_A, "refresh-a"),
    "code-other-tenant": (TENANT_B, TENANT_B, "refresh-b"),
    "code-forged-issuer": (TENANT_A, TENANT_B, "refresh-forged"),
    "code-no-id-token": (None, None, "refresh-none"),
    "code-no-token-type": (TENANT_A, TENANT_A, "refresh-untyped"),
}
REFRESHES = {
    "refresh-a": (TENANT_A, TENANT_A, "refresh-a2"),
    "refresh-a2": (TENANT_A, TENANT_A, "refresh-a2"),
    "refresh-switch": (TENANT_B, TENANT_B, "refresh-switch"),
    "refresh-no-id-token": (None, None, "refresh-a2"),
}
issued = 0


def b64(data):
    return base64.urlsafe_b64encode(data).rstrip(b"=").decode()


def id_token(tid, iss_tenant):
    header = {"alg": "RS256", "typ": "JWT", "kid": "test"}
    claims = {"iss": f"{origin}/{iss_tenant}/v2.0", "aud": CLIENT, "tid": tid, "oid": OID,
              "preferred_username": "ana@contoso.example", "exp": int(time.time()) + 3600}
    return ".".join([b64(json.dumps(header).encode()), b64(json.dumps(claims).encode()), b64(b"test-signature")])


class Handler(http.server.BaseHTTPRequestHandler):
    def log_message(self, *args):
        pass

    def send_json(self, obj, status=200):
        data = json.dumps(obj).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_POST(self):
        global issued
        body = self.rfile.read(int(self.headers.get("Content-Length", 0))).decode()
        if not TOKEN_PATH.match(self.path):
            return self.send_json({}, 404)
        fields = urllib.parse.parse_qs(body)

        def one(name):
            return (fields.get(name) or [None])[0]

        if "resource" in fields:
            return self.send_json({"error": "invalid_request", "error_description": "AADSTS901002"}, 400)
        if one("client_id") != CLIENT or one("client_secret") != SECRET:
            return self.send_json({"error": "invalid_client"}, 401)
        scopes = (one("scope") or "").split()
        if "offline_access" not in scopes or "openid" not in scopes:
            return self.send_json({"error": "invalid_scope"}, 400)
        if one("grant_type") == "authorization_code":
            if one("code_verifier") != "test-verifier" or not one("redirect_uri"):
                return self.send_json({"error": "invalid_grant"}, 400)
            outcome = CODES.get(one("code"))
        elif one("grant_type") == "refresh_token":
            outcome = REFRESHES.get(one("refresh_token"))
        else:
            return self.send_json({"error": "unsupported_grant_type"}, 400)
        if outcome is None:
            return self.send_json({"error": "invalid_grant"}, 400)
        tid, iss_tenant, refresh = outcome
        issued += 1
        response = {"access_token": f"graph-access-{issued}", "token_type": "Bearer", "expires_in": 3600,
                    "scope": " ".join(scopes), "refresh_token": refresh}
        if tid is not None:
            response["id_token"] = id_token(tid, iss_tenant)
        if one("code") == "code-no-token-type":
            del response["token_type"]
        return self.send_json(response)


server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
origin = "https://localhost:" + str(server.server_port)
context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
context.load_cert_chain(root / "server.pem", root / "server.key")
server.socket = context.wrap_socket(server.socket, server_side=True)
print(origin, flush=True)
server.serve_forever()
