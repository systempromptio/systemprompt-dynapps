#!/usr/bin/env python3
"""A request body shaped like a Claude Code turn, for run.sh --body claude-code.

  claude_code_body.py [--tools 60] [--pairs 50] [--system-kb 30]
A ~30 KB system prompt, 60 tool definitions identical for every user (so every
request shares one tool catalog row), and 50 tool_use/tool_result pairs of
history before the final user turn. Deterministic and synthetic: no real data.
Standard library only.
"""
import argparse
import json


def tool(i):
    return {
        "name": f"tool_{i:02d}",
        "description": f"Synthetic load-test tool {i}: reads, edits or searches files in the workspace.",
        "input_schema": {
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Workspace-relative path."},
                "pattern": {"type": "string", "description": "Search pattern."},
                "limit": {"type": "integer", "minimum": 1, "maximum": 500},
            },
            "required": ["path"],
        },
    }


def history(pairs, tools):
    messages = [{"role": "user", "content": "Refactor the billing module and keep the tests green."}]
    for i in range(pairs):
        tid = f"toolu_{i:04d}"
        messages.append({
            "role": "assistant",
            "content": [
                {"type": "text", "text": f"Step {i}: inspecting the next file."},
                {"type": "tool_use", "id": tid, "name": f"tool_{i % tools:02d}",
                 "input": {"path": f"src/billing/module_{i}.rs", "limit": 200}},
            ],
        })
        messages.append({
            "role": "user",
            "content": [
                {"type": "tool_result", "tool_use_id": tid,
                 "content": f"fn charge_{i}(amount: u64) -> Result<Receipt, Error> {{ /* {i} */ }}\n" * 4},
            ],
        })
    messages.append({"role": "user", "content": "Continue."})
    return messages


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--tools", type=int, default=60)
    ap.add_argument("--pairs", type=int, default=50)
    ap.add_argument("--system-kb", type=int, default=30)
    a = ap.parse_args()
    line = "You are a coding agent working in a governed workspace. Follow the repository rules.\n"
    system = (line * (a.system_kb * 1024 // len(line) + 1))[: a.system_kb * 1024]
    print(json.dumps({
        "model": "claude-sonnet-5-5",
        "max_tokens": 32,
        "system": system,
        "tools": [tool(i) for i in range(a.tools)],
        "messages": history(a.pairs, a.tools),
    }))


if __name__ == "__main__":
    main()
