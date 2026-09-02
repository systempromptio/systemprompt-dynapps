#!/usr/bin/env python3
"""Generate the value-architecture diagram as both an editable draw.io file and an SVG.

One model, two emitters, so the file the platform team edits and the file the
documentation serves cannot drift. Run from the repository root:

    python3 scripts/make-value-architecture.py

Writes docs/architecture/value-architecture.drawio and
storage/files/images/architecture/value-architecture.svg. Deterministic: rerunning
with no source change leaves both files byte-identical.
"""

from pathlib import Path
from xml.sax.saxutils import escape

# Palette: the astound product set (storage/files/css/core/variables.css).
# Not the orange print set used by core's internal architecture factsheets.
INK = "#171717"
SECONDARY = "#4d4d4d"
MUTED = "#707070"
BG = "#f8f7f3"
SURFACE = "#f1efed"
WHITE = "#ffffff"
BORDER = "#dddddd"
BORDER_SOFT = "#e7e5e1"
ACCENT = "#fe190a"
BLUE = "#0071ce"

W, H = 1560, 1000
FONT = "Inter, -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif"
MONO = "'SF Mono', ui-monospace, 'Cascadia Mono', Menlo, monospace"

CP_X, CP_Y, CP_W, CP_H = 264, 96, 924, 470
DP_X, DP_Y, DP_W, DP_H = 264, 616, 924, 104
IN_X, IN_Y, IN_W, IN_H = 264, 756, 924, 180
PR_X, PR_Y, PR_W, PR_H = 1212, 96, 324, 880

BANDS = [
    {
        "id": "actors",
        "x": 24, "y": 96, "w": 176, "h": 268,
        "label": "USERS & AGENTS",
        "fill": SURFACE, "stroke": BORDER, "dashed": True, "label_color": MUTED,
    },
    {
        "id": "providers",
        "x": 24, "y": 396, "w": 176, "h": 222,
        "label": "MODEL PROVIDERS",
        "fill": SURFACE, "stroke": BORDER, "dashed": True, "label_color": MUTED,
    },
    {
        "id": "controlplane",
        "x": CP_X, "y": CP_Y, "w": CP_W, "h": CP_H,
        "label": "CONTROL PLANE  ·  systemprompt",
        "fill": WHITE, "stroke": ACCENT, "dashed": False, "label_color": ACCENT,
        "sub": "Every decision made per request, in the request path. This is the product.",
    },
    {
        "id": "dataplane",
        "x": DP_X, "y": DP_Y, "w": DP_W, "h": DP_H,
        "label": "DATA PLANE  ·  your cloud",
        "fill": SURFACE, "stroke": BORDER, "dashed": True, "label_color": MUTED,
        "sub": "Provisioned and owned by your platform team. systemprompt neither owns nor replaces any of it.",
    },
    {
        "id": "inset",
        "x": IN_X, "y": IN_Y, "w": IN_W, "h": IN_H,
        "label": "WHY THIS CANNOT LIVE IN THE LAYER BELOW",
        "fill": WHITE, "stroke": BORDER, "dashed": False, "label_color": INK,
    },
    {
        "id": "principles",
        "x": PR_X, "y": PR_Y, "w": PR_W, "h": PR_H,
        "label": "CORE PRINCIPLES",
        "fill": WHITE, "stroke": BORDER, "dashed": False, "label_color": INK,
    },
]

CARDS = [
    {
        "id": "identity", "col": 0, "row": 0,
        "title": "Identity & entitlement",
        "lines": [
            "SSO group claim is the subject",
            "Groups → projects → authorization rules",
            "Operators hold passkeys, not passwords",
            "No account model of our own",
        ],
    },
    {
        "id": "policy", "col": 1, "row": 0,
        "title": "Policy · four synchronous stages",
        "lines": [
            ("1  Scope check", "the subject may call this"),
            ("2  Secret scan", "35+ patterns, both directions"),
            ("3  Blocklist", "tool and destination policy"),
            ("4  Rate limit", "per-subject quota and budget"),
        ],
        "accent": True,
    },
    {
        "id": "tools", "col": 2, "row": 0,
        "title": "Tool & capability distribution",
        "lines": [
            "MCP servers, brokered and governed",
            "Marketplace plugins and skills",
            "Synced to every client, centrally",
            "One catalogue, not per-laptop config",
        ],
    },
    {
        "id": "routing", "col": 0, "row": 1,
        "title": "Model routing",
        "lines": [
            "Provider and model chosen by policy",
            "Region and data residency enforced",
            "Fallbacks and failure handling",
            "One wire shape across providers",
        ],
    },
    {
        "id": "cost", "col": 1, "row": 1,
        "title": "Cost & attribution",
        "lines": [
            "Spend resolved to subject, agent, tool",
            "Budgets enforced before the call, not after",
            "Per-project and per-group rollups",
        ],
    },
    {
        "id": "audit", "col": 2, "row": 1,
        "title": "Audit & evidence",
        "lines": [
            "One trace_id per request, end to end",
            "identity → agent → tool → result → cost",
            "Denials recorded with the policy that fired",
            "Immutable, queryable, exportable",
        ],
    },
]

ACTORS = [
    {"id": "engineers", "label": "Project engineers", "sub": "Claude Code, terminal"},
    {"id": "business", "label": "Business users", "sub": "Cowork, desktop bridge"},
    {"id": "agents", "label": "Agents", "sub": "A2A, scheduled, autonomous"},
]

PROVIDERS = [
    {"id": "anthropic", "label": "Anthropic"},
    {"id": "openai", "label": "OpenAI"},
    {"id": "gemini", "label": "Google Gemini"},
    {"id": "bedrock", "label": "Bedrock, others"},
]

DATA_PLANE = [
    "WAF", "Load balancer", "NAT egress", "Containers",
    "Postgres", "Vault", "Object store", "Logging", "Terraform",
]

INSET_ROWS = [
    ("WAF", "inspects HTTP", "cannot see an exfiltration inside a legitimate tool call"),
    ("IAM", "authenticates humans to the cloud", "cannot express “this agent, acting as this user, on this project”"),
    ("Vault", "stores a secret", "cannot notice one leaving in a model response"),
    ("Monitoring", "counts requests and latency", "cannot attribute $3.40 to a person, an agent and a tool call"),
]

PRINCIPLES = [
    "One binary, one database. No control-plane SaaS to depend on.",
    "Governance in the request path, not beside it. A report cannot block a call.",
    "Identity comes from your IdP, never from us.",
    "Deny by default. An unknown tool is not an allowed tool.",
    "Every decision audited under one trace_id — including the denials.",
    "Open standards: MCP, A2A, and the OpenAI and Anthropic wire formats.",
    "Vendor agnostic. No cloud provider API appears in the source.",
    "Self-hosted. Prompts, responses and audit stay in your tenancy.",
    "Immutable image, versioned schema. Deploys are a tag change.",
]

EDGES = [
    {"id": "e_in", "src": "actors", "dst": "controlplane", "label": "every request",
     "x1": 200, "y1": 230, "x2": CP_X, "y2": 230, "color": ACCENT},
    {"id": "e_out", "src": "controlplane", "dst": "providers", "label": "one egress path",
     "x1": CP_X, "y1": 512, "x2": 200, "y2": 512, "color": BLUE},
    {"id": "e_gov", "src": "controlplane", "dst": "dataplane", "label": "governs",
     "x1": 620, "y1": CP_Y + CP_H, "x2": 620, "y2": DP_Y, "color": INK},
    {"id": "e_run", "src": "dataplane", "dst": "controlplane", "label": "runs on",
     "x1": 840, "y1": DP_Y, "x2": 840, "y2": CP_Y + CP_H, "color": MUTED},
]

CARD_W, CARD_H = 284, 168
CARD_X0, CARD_Y0 = CP_X + 20, CP_Y + 76
CARD_GAP_X, CARD_GAP_Y = 18, 18


def card_box(card):
    x = CARD_X0 + card["col"] * (CARD_W + CARD_GAP_X)
    y = CARD_Y0 + card["row"] * (CARD_H + CARD_GAP_Y)
    return x, y, CARD_W, CARD_H


def actor_box(i):
    return 40, 148 + i * 72, 144, 58


def provider_box(i):
    return 40, 444 + i * 44, 144, 34


def dp_box(i):
    return DP_X + 20 + i * 100, DP_Y + 46, 96, 38


def inset_row_y(i):
    return IN_Y + 58 + i * 26


# --------------------------------------------------------------------------- SVG


def t(x, y, s, size=12, weight=400, fill=INK, anchor="start", family=FONT, spacing=None):
    ls = f' letter-spacing="{spacing}"' if spacing else ""
    return (
        f'<text x="{x}" y="{y}" font-family="{family}" font-size="{size}" '
        f'font-weight="{weight}" fill="{fill}" text-anchor="{anchor}"{ls}>{escape(s)}</text>'
    )


def rect(x, y, w, h, fill, stroke, dashed=False, rx=6, width=1):
    dash = ' stroke-dasharray="6 4"' if dashed else ""
    return (
        f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{rx}" fill="{fill}" '
        f'stroke="{stroke}" stroke-width="{width}"{dash}/>'
    )


def arrow(x1, y1, x2, y2, color, label, marker):
    mid_x, mid_y = (x1 + x2) / 2, (y1 + y2) / 2
    parts = [
        f'<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{color}" '
        f'stroke-width="1.5" marker-end="url(#{marker})"/>'
    ]
    if x1 == x2:
        parts.append(t(mid_x + 8, mid_y + 4, label, 11, 600, color))
    else:
        parts.append(t(mid_x, mid_y - 9, label, 9.5, 700, color, anchor="middle"))
    return "".join(parts)


def emit_svg():
    o = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" '
        f'height="{H}" role="img" aria-label="systemprompt value architecture: the '
        f'control plane above your cloud data plane">',
        "<defs>",
    ]
    for name, color in (("ar-accent", ACCENT), ("ar-blue", BLUE), ("ar-ink", INK), ("ar-muted", MUTED)):
        o.append(
            f'<marker id="{name}" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" '
            f'markerHeight="7" orient="auto-start-reverse">'
            f'<path d="M 0 0 L 10 5 L 0 10 z" fill="{color}"/></marker>'
        )
    o.append("</defs>")
    o.append(rect(0, 0, W, H, BG, BG, rx=0))

    o.append(t(24, 44, "systemprompt · value architecture", 20, 700, INK))
    o.append(t(24, 68, "The same deployment, projected by what makes decisions rather than by what is provisioned.", 12, 400, SECONDARY))

    for b in BANDS:
        o.append(rect(b["x"], b["y"], b["w"], b["h"], b["fill"], b["stroke"], b["dashed"],
                      width=1.5 if b["id"] == "controlplane" else 1))
        o.append(t(b["x"] + 14, b["y"] + 22, b["label"], 11, 700, b["label_color"], spacing="0.8"))
        if b.get("sub"):
            o.append(t(b["x"] + 14, b["y"] + 40, b["sub"], 11, 400, MUTED))

    for i, a in enumerate(ACTORS):
        x, y, w, h = actor_box(i)
        o.append(rect(x, y, w, h, WHITE, BORDER))
        o.append(t(x + 12, y + 24, a["label"], 12, 700, INK))
        o.append(t(x + 12, y + 42, a["sub"], 10, 400, MUTED))

    for i, p in enumerate(PROVIDERS):
        x, y, w, h = provider_box(i)
        o.append(rect(x, y, w, h, WHITE, BORDER))
        o.append(t(x + 12, y + 22, p["label"], 11, 500, SECONDARY))

    for c in CARDS:
        x, y, w, h = card_box(c)
        stroke = ACCENT if c.get("accent") else BORDER
        o.append(rect(x, y, w, h, WHITE, stroke, width=1.5 if c.get("accent") else 1))
        o.append(f'<rect x="{x}" y="{y}" width="4" height="{h}" rx="2" fill="{ACCENT if c.get("accent") else BORDER_SOFT}"/>')
        o.append(t(x + 18, y + 28, c["title"], 13, 700, INK))
        for j, line in enumerate(c["lines"]):
            ly = y + 52 + j * 21
            if isinstance(line, tuple):
                o.append(t(x + 18, ly, line[0], 10.5, 700, INK, family=MONO))
                o.append(t(x + 122, ly, line[1], 10.5, 400, SECONDARY))
            else:
                o.append(t(x + 18, ly, line, 11, 400, SECONDARY))

    for i, label in enumerate(DATA_PLANE):
        x, y, w, h = dp_box(i)
        o.append(rect(x, y, w, h, WHITE, BORDER, rx=4))
        o.append(t(x + w / 2, y + 24, label, 10, 500, MUTED, anchor="middle"))

    for i, (subject, does, cannot) in enumerate(INSET_ROWS):
        y = inset_row_y(i)
        o.append(t(IN_X + 22, y, subject, 12, 700, INK))
        o.append(t(IN_X + 130, y, does, 11, 400, MUTED))
        o.append(t(IN_X + 360, y, "→", 11, 400, MUTED))
        o.append(t(IN_X + 386, y, cannot, 11, 500, INK))
    o.append(t(IN_X + 22, IN_Y + IN_H - 18,
               "Each pair looks redundant on a topology diagram. It is not: the lower layer is blind to agent semantics.",
               11, 400, SECONDARY))

    for i, p in enumerate(PRINCIPLES):
        y = PR_Y + 60 + i * 76
        o.append(t(PR_X + 22, y, str(i + 1), 12, 700, ACCENT))
        words, line, lines = p.split(" "), "", []
        for word in words:
            if len(line) + len(word) + 1 > 38:
                lines.append(line)
                line = word
            else:
                line = f"{line} {word}".strip()
        lines.append(line)
        for j, text_line in enumerate(lines[:3]):
            o.append(t(PR_X + 44, y + j * 17, text_line, 11.5, 400, SECONDARY))

    o.append(t(PR_X + 22, PR_Y + PR_H - 42, "The layer below is provisioned once.", 11, 400, MUTED))
    o.append(t(PR_X + 22, PR_Y + PR_H - 24, "The layer above decides on every call.", 11, 700, INK))

    markers = {ACCENT: "ar-accent", BLUE: "ar-blue", INK: "ar-ink", MUTED: "ar-muted"}
    for e in EDGES:
        o.append(arrow(e["x1"], e["y1"], e["x2"], e["y2"], e["color"], e["label"], markers[e["color"]]))

    o.append("</svg>")
    return "\n".join(o) + "\n"


# ------------------------------------------------------------------------ drawio


def mx_value(s):
    return escape(s).replace("\n", "&lt;br&gt;")


CELL_ID = {}


def cell(cid, value, x, y, w, h, style):
    CELL_ID[cid] = cid
    return (
        f'        <mxCell id="{cid}" value="{mx_value(value)}" style="{escape(style)}" '
        f'vertex="1" parent="1">\n'
        f'          <mxGeometry x="{x}" y="{y}" width="{w}" height="{h}" as="geometry"/>\n'
        f'        </mxCell>'
    )


def edge_cell(cid, value, x1, y1, x2, y2, color):
    style = (
        f"endArrow=block;endFill=1;html=1;rounded=0;strokeColor={color};strokeWidth=1.5;"
        f"fontColor={color};fontSize=11;fontStyle=1;labelBackgroundColor={BG};"
    )
    return (
        f'        <mxCell id="{cid}" value="{mx_value(value)}" style="{escape(style)}" '
        f'edge="1" parent="1">\n'
        f'          <mxGeometry relative="1" as="geometry">\n'
        f'            <mxPoint x="{x1}" y="{y1}" as="sourcePoint"/>\n'
        f'            <mxPoint x="{x2}" y="{y2}" as="targetPoint"/>\n'
        f'          </mxGeometry>\n'
        f'        </mxCell>'
    )


def box_style(fill, stroke, dashed=False, width=1, font_color=INK, size=11, bold=True, top=True):
    return (
        f"rounded=1;arcSize=6;whiteSpace=wrap;html=1;fillColor={fill};strokeColor={stroke};"
        f"strokeWidth={width};dashed={1 if dashed else 0};"
        f"verticalAlign={'top' if top else 'middle'};align=left;spacingLeft=12;spacingTop=6;"
        f"fontColor={font_color};fontSize={size};fontStyle={1 if bold else 0};"
        f"fontFamily=Inter;"
    )


def emit_drawio():
    cells = []

    for b in BANDS:
        value = b["label"]
        if b.get("sub"):
            value = f"{b['label']}\n{b['sub']}"
        cells.append(cell(
            b["id"], value, b["x"], b["y"], b["w"], b["h"],
            box_style(b["fill"], b["stroke"], b["dashed"],
                      width=1.5 if b["id"] == "controlplane" else 1,
                      font_color=b["label_color"]),
        ))

    for i, a in enumerate(ACTORS):
        x, y, w, h = actor_box(i)
        cells.append(cell(a["id"], f"{a['label']}\n{a['sub']}", x, y, w, h,
                          box_style(WHITE, BORDER, size=11)))

    for i, p in enumerate(PROVIDERS):
        x, y, w, h = provider_box(i)
        cells.append(cell(p["id"], p["label"], x, y, w, h,
                          box_style(WHITE, BORDER, size=11, bold=False, top=False)))

    for c in CARDS:
        x, y, w, h = card_box(c)
        stroke = ACCENT if c.get("accent") else BORDER
        body = "\n".join(
            f"{ln[0]}  {ln[1]}" if isinstance(ln, tuple) else ln for ln in c["lines"]
        )
        cells.append(cell(c["id"], f"<b>{escape(c['title'])}</b>\n{body}", x, y, w, h,
                          box_style(WHITE, stroke, width=1.5 if c.get("accent") else 1,
                                    font_color=SECONDARY, size=11, bold=False)))

    for i, label in enumerate(DATA_PLANE):
        x, y, w, h = dp_box(i)
        cells.append(cell(f"dp{i}", label, x, y, w, h,
                          box_style(WHITE, BORDER, font_color=MUTED, size=10,
                                    bold=False, top=False)))

    for i, (subject, does, cannot) in enumerate(INSET_ROWS):
        y = inset_row_y(i) - 14
        cells.append(cell(f"in{i}", f"<b>{escape(subject)}</b> {escape(does)} → {escape(cannot)}",
                          IN_X + 16, y, IN_W - 32, 24,
                          box_style("none", "none", font_color=INK, size=11, bold=False,
                                    top=False)))
    cells.append(cell("in_note",
                      "Each pair looks redundant on a topology diagram. It is not: the lower layer is blind to agent semantics.",
                      IN_X + 16, IN_Y + IN_H - 32, IN_W - 32, 22,
                      box_style("none", "none", font_color=SECONDARY, size=11, bold=False,
                                top=False)))

    for i, p in enumerate(PRINCIPLES):
        y = PR_Y + 46 + i * 76
        cells.append(cell(f"pr{i}", f"<b>{i + 1}</b>  {escape(p)}", PR_X + 16, y,
                          PR_W - 32, 56,
                          box_style("none", "none", font_color=SECONDARY, size=11,
                                    bold=False, top=True)))
    cells.append(cell("pr_note",
                      "The layer below is provisioned once.\n<b>The layer above decides on every call.</b>",
                      PR_X + 16, PR_Y + PR_H - 60, PR_W - 32, 44,
                      box_style("none", "none", font_color=MUTED, size=11, bold=False)))

    cells.append(cell("title",
                      "<b>systemprompt · value architecture</b>\nThe same deployment, projected by what makes decisions rather than by what is provisioned.",
                      24, 24, 900, 52,
                      box_style("none", "none", font_color=INK, size=14, bold=False)))

    for e in EDGES:
        cells.append(edge_cell(e["id"], e["label"], e["x1"], e["y1"], e["x2"], e["y2"], e["color"]))

    body = "\n".join(cells)
    return (
        '<mxfile host="systemprompt" type="device">\n'
        '  <diagram id="value-architecture" name="Value architecture">\n'
        f'    <mxGraphModel dx="{W}" dy="{H}" grid="0" gridSize="10" guides="1" tooltips="1" '
        'connect="1" arrows="1" fold="1" page="1" pageScale="1" pageWidth="1600" '
        f'pageHeight="1100" math="0" shadow="0" background="{BG}">\n'
        '      <root>\n'
        '        <mxCell id="0"/>\n'
        '        <mxCell id="1" parent="0"/>\n'
        f'{body}\n'
        '      </root>\n'
        '    </mxGraphModel>\n'
        '  </diagram>\n'
        '</mxfile>\n'
    )


def main():
    root = Path(__file__).resolve().parent.parent
    svg_path = root / "storage/files/images/architecture/value-architecture.svg"
    drawio_path = root / "docs/architecture/value-architecture.drawio"
    svg_path.parent.mkdir(parents=True, exist_ok=True)
    drawio_path.parent.mkdir(parents=True, exist_ok=True)
    svg_path.write_text(emit_svg(), encoding="utf-8")
    drawio_path.write_text(emit_drawio(), encoding="utf-8")
    print(f"{svg_path.relative_to(root)} written")
    print(f"{drawio_path.relative_to(root)} written")


if __name__ == "__main__":
    main()
