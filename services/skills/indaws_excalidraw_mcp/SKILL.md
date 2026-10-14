---
name: indaws-excalidraw-mcp
version: 1.0.4
effort: medium
model: sonnet
description: Create and edit Excalidraw diagrams through an MCP server. Use this skill when you need to create visual diagrams, flowcharts, architecture drawings, wireframes, or any visual content using Excalidraw. Provides 23 tools for element creation, layout, grouping, export, and real-time canvas manipulation. Requires one-time local setup (Node.js MCP server).
---

# Excalidraw MCP

Create and manipulate Excalidraw diagrams programmatically through the MCP protocol. This skill enables Claude to draw diagrams, flowcharts, architecture schemas, wireframes, and any visual content on a live Excalidraw canvas.

## Capabilities

| Category | Tools | Description |
|----------|-------|-------------|
| **Elements** | create, read, update, delete, batch | Create and manipulate shapes, text, arrows, lines |
| **Layout** | align, distribute, group, lock | Organize elements on the canvas |
| **Vision** | inspect, screenshot | AI can see and analyze the canvas state |
| **File I/O** | export, import, mermaid | Export/import `.excalidraw` JSON, convert Mermaid diagrams |
| **State** | snapshot, rollback, clear | Canvas state management and undo |

## Prerequisites

- **Node.js 18+** installed
- **npm** installed

## Setup (One-Time)

Run the automated setup script (from the skill directory):

```bash
bash skills/excalidraw-mcp/scripts/setup.sh
```

This will:
1. Clone the `mcp_excalidraw` repository to `~/Agents/mcp_excalidraw`
2. Install dependencies and build the project
3. Register the MCP server in your agent's MCP configuration (see below)

### Manual Setup

```bash
# 1. Clone and build
git clone https://github.com/yctimlin/mcp_excalidraw.git ~/Agents/mcp_excalidraw
cd ~/Agents/mcp_excalidraw
npm ci && npm run build

# 2. Register the server in your agent's MCP configuration (see below)
```

Register the server with your agent. The **server definition** is the same everywhere; only the file it goes in changes — in Claude Code it is `~/.claude/.mcp.json`, in opencode the `mcp` block of `opencode.json`. Check your agent's documentation if you use another one.

Server definition:

```json
{
  "mcpServers": {
    "excalidraw": {
      "command": "node",
      "args": ["<HOME>/Agents/mcp_excalidraw/dist/index.js"],
      "env": {
        "EXPRESS_SERVER_URL": "http://localhost:3000"
      }
    }
  }
}
```

Replace `<HOME>` with your actual home directory path.

## Usage

### 1. Start the Canvas Server

Before using Excalidraw tools, start the canvas web server in a separate terminal:

```bash
cd ~/Agents/mcp_excalidraw && npm run canvas
```

This opens a browser with the Excalidraw canvas at `http://localhost:3000`.

### 2. Use via Claude

Once the canvas server is running and the MCP is configured, Claude has access to all Excalidraw tools. Examples:

- "Draw a flowchart showing the user login process"
- "Create an architecture diagram for our microservices"
- "Make a wireframe for the dashboard page"
- "Convert this Mermaid diagram to Excalidraw"
- "Export the current canvas as an .excalidraw file"

### 3. Canvas Server Port

The default port is `3000`. To use a different port:

```bash
PORT=3001 npm run canvas
```

Update `EXPRESS_SERVER_URL` in your `.mcp.json` accordingly.

## Troubleshooting

| Problem | Solution |
|---------|----------|
| "MCP server not found" | Ensure your agent's MCP configuration points at the correct absolute path to `dist/index.js` |
| "Connection refused" | Start the canvas server first: `cd ~/Agents/mcp_excalidraw && npm run canvas` |
| Build fails | Ensure Node.js 18+ is installed: `node --version` |
| Port conflict | Use a different port: `PORT=3001 npm run canvas` |

## Source

- **Repository**: https://github.com/yctimlin/mcp_excalidraw
- **License**: MIT

## GOTCHAS
1. **MCP server must be running locally** — skill fails silently if server is down. Check with `mcp__excalidraw__ping`.
2. **Canvas size has no auto-fit** — elements placed beyond viewport are invisible. Use layout tools to center.
3. **Group IDs are ephemeral** — don't store group IDs between sessions. Re-query with `read_elements`.
4. **Arrow connections break on move** — if you move a connected element, re-run `create_arrow` to reconnect.
5. **Export resolution is fixed** — PNG export is 2x viewport. For higher res, increase canvas size before export.

## Workflow
1. Plan diagram layout (elements, connections, grouping)
2. Create elements with `create_element` / `batch_create`
3. Arrange with `align_elements` / `distribute_elements`
4. Connect with arrows
5. Group related elements
6. Export as PNG or SVG

## Delegates to
None — this is a leaf skill.

## Consumed by
See DEPENDENCIES.md for cross-suite consumers. For shared patterns, see `common:references/shared/`.

## RULES
### NEVER
- Create elements without planning layout first — repositioning is expensive
- Assume MCP server is running — always verify connection
### ALWAYS
- Use batch operations for 3+ elements
- Center the final diagram in viewport
