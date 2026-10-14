# Environment Detection

Detect the user's OS, shell, and terminal to adapt commands and install instructions.

## Detection commands

Run these at startup. Store results for the session.

### OS detection

```bash
uname -s
# Darwin → macOS
# Linux  → Linux
# MINGW* / MSYS* / CYGWIN* → Windows (Git Bash)
```

If `uname` is not available (pure Windows CMD/PowerShell), fall back:

```powershell
$env:OS
# Windows_NT → Windows
```

### Shell detection

```bash
echo $SHELL
# /bin/zsh    → zsh (macOS default)
# /bin/bash   → bash (Linux default)
# /usr/bin/fish → fish

# For current shell (may differ from $SHELL):
echo $0
# -zsh, bash, fish
```

PowerShell detection:

```powershell
$PSVersionTable.PSVersion
# If this works → PowerShell
```

### Terminal emulator

```bash
echo $TERM_PROGRAM
# iTerm.app        → iTerm2 (macOS)
# Apple_Terminal    → Terminal.app (macOS)
# vscode           → VS Code integrated terminal
# tmux             → tmux multiplexer

echo $WT_SESSION
# If set → Windows Terminal
```

## Install check pattern

Before running any external tool, always verify it exists. This is the standard pattern:

```
1. which {tool}
2. If NOT found:
   → Ask: "No tienes {tool} instalado. ¿Quieres que lo instale yo o prefieres hacerlo tú?"
   → If yes: run the install command for the detected OS (see table below)
   → If no: show the command and stop
3. If found: proceed
```

This applies to: `gh`, `glab`, `pre-commit`, and any other CLI the skill needs.

## Install command mapping

| Package | Check | macOS | Linux (Debian/Ubuntu) | Windows |
|---------|-------|-------|----------------------|---------|
| `glab` | `which glab` | `brew install glab` | See note below | `winget install GLab.GLab` |
| `gh` | `which gh` | `brew install gh` | `sudo apt install gh` | `winget install GitHub.cli` |
| `pre-commit` | `which pre-commit` | `brew install pre-commit` | `pip install pre-commit` | `pip install pre-commit` |
| `git` | `which git` | `xcode-select --install` | `sudo apt install git` | `winget install Git.Git` |

**Linux glab install** (no apt package):

```bash
# Option 1: Download binary
curl -sL https://github.com/profclems/glab/releases/latest/download/glab_*_Linux_x86_64.tar.gz | tar xz
sudo mv glab /usr/local/bin/

# Option 2: Homebrew on Linux
brew install glab
```

## Known terminal limitations

| Terminal | Limitation | Workaround |
|----------|-----------|------------|
| Windows CMD | Path length limit (260 chars) | Use PowerShell or Git Bash |
| Windows CMD | No ANSI colors by default | Use Windows Terminal instead |
| PowerShell | Different quoting rules (`"` vs `'`) | Prefer single quotes for git commands |
| VS Code terminal | Sometimes truncates long output | Use `--no-pager` with git commands |
| tmux | `glab ci view` may fail (TTY issues) | Use `glab ci status` instead |

## Usage pattern

At startup, detect once and store:

```
Environment:
  OS: macOS (Darwin)
  Shell: zsh
  Terminal: VS Code
  Provider CLI: glab (installed, authenticated)
```

Use this to:
1. Select the right install command when a CLI is missing
2. Adapt shell syntax in examples and commands
3. Warn about known terminal limitations before they cause errors
