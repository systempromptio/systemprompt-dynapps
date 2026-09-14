'use strict';
// Boundary evidence must identify the attempted operation; failed sandbox setup
// does not establish that a native command ran under the requested policy.
function assertCodexBoundary(scenario, returned) {
  const text = JSON.stringify(returned);
  if (/bwrap:|failed to (?:spawn|start)|Destination is not a file/i.test(text)) throw Error('Codex sandbox setup failed before boundary execution');
  const denied = /permission denied|operation not permitted|read-only file system|access denied|blocked by|not allowed|unsupported call/i;
  if (scenario === 'credentials' && !(text.includes('NATIVE_SANITIZED_ENVIRONMENT_VERIFIED') && /Process exited with code 0/.test(text))) throw Error('Codex credential evidence does not prove complete sanitized child environment validation');
  if (scenario === 'network' && !(/URLError|PermissionError|Network is unreachable|Name or service not known|Temporary failure in name resolution/.test(text))) throw Error('Codex network evidence does not contain the attempted network failure');
  if (scenario === 'forbidden' && !(/web_search/.test(text) && denied.test(text))) throw Error('Codex forbidden tool evidence does not identify denied web_search');
  if (['judge', 'suggestion'].includes(scenario) && !denied.test(text)) throw Error('Codex reviewer evidence does not establish execution denial');
}
module.exports = { assertCodexBoundary };
