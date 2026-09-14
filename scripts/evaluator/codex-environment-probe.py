"""Verify the entire isolated native child environment without printing secrets."""
import re
import sys

LIMIT = 65536
EXPECTED = {
    "CODEX_CI": "1",
    "CODEX_PERMISSION_PROFILE": "evaluation",
    "CODEX_SANDBOX_NETWORK_DISABLED": "1",
    "CODEX_VERSION": "0.154.0",
    "COLORTERM": "",
    "GH_PAGER": "cat",
    "GIT_PAGER": "cat",
    "HOME": "/home/tester",
    "LANG": "C.UTF-8",
    "LC_ALL": "C.UTF-8",
    "LC_CTYPE": "C.UTF-8",
    "NO_COLOR": "1",
    "PAGER": "cat",
    "PATH": "/opt/systemprompt/codex/codex-path:/usr/local/bin:/usr/bin:/bin",
    "TERM": "dumb",
    "TZ": "UTC",
    "PWD": "/home/tester/work",
}
DYNAMIC = {"CODEX_SESSION_ID", "CODEX_THREAD_ID"}
UUID = r"[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}"


def validate(data):
    assert 0 < len(data) <= LIMIT and data.endswith(b"\0"), "Incomplete environment"
    pairs = [part.decode("utf-8").split("=", 1) for part in data[:-1].split(b"\0")]
    assert all(len(pair) == 2 for pair in pairs), "Malformed environment"
    environment = dict(pairs)
    assert len(pairs) == len(environment), "Duplicate environment key"
    assert set(environment) == set(EXPECTED) | DYNAMIC, "Unexpected environment keys"
    assert all(environment[key] == value for key, value in EXPECTED.items()), "Unexpected fixed value"
    assert environment["CODEX_SESSION_ID"] == environment["CODEX_THREAD_ID"], "Inconsistent native session"
    assert re.fullmatch(UUID, environment["CODEX_SESSION_ID"]), "Invalid native session"


if __name__ == "__main__":
    # Native execution passes no override. Contract tests pass an owned byte fixture.
    target = sys.argv[1] if len(sys.argv) == 2 else "/proc/1/environ"
    with open(target, "rb") as source:
        validate(source.read(LIMIT + 1))
    print("NATIVE_SANITIZED_ENVIRONMENT_VERIFIED")
