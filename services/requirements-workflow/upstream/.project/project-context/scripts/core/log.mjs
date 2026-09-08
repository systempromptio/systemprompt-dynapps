// All diagnostic output goes to stderr so it never corrupts the MCP stdio
// JSON-RPC stream on stdout.

export function log(...args) {
  process.stderr.write(`${args.join(' ')}\n`);
}

export function warn(...args) {
  process.stderr.write(`[warn] ${args.join(' ')}\n`);
}

export function error(...args) {
  process.stderr.write(`[error] ${args.join(' ')}\n`);
}
