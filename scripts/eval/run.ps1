# One headless Claude Code run on Windows through the Astound bridge's local
# proxy, so the conversation is governed and recorded by the gateway exactly
# like a user's. Reads the prompt from a file (no argv quoting) and writes the
# claude -p JSON result next to it.
param([string]$PromptFile, [string]$OutFile, [string]$Model)
$env:CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS = '1'
$env:ANTHROPIC_BASE_URL = 'http://127.0.0.1:48217'
$env:ANTHROPIC_AUTH_TOKEN = (Get-Content "$env:APPDATA\astound\bridge-loopback.key" -Raw).Trim()
Set-Location "$env:USERPROFILE\sp-skill-runs"
$prompt = Get-Content $PromptFile -Raw -Encoding UTF8
"" | & "$env:USERPROFILE\.local\bin\claude.exe" -p $prompt --model $Model --output-format json --dangerously-skip-permissions --max-turns 40 |
    Out-File -Encoding utf8 $OutFile
