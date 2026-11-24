// Part four of the page runtime (see runtime-core.js): runs the other three.
// The frame route calls this function expression with the text of each part
// and the host's `{origins, contract}`. Core answers null when the page
// already has a `governance`, and then nothing else runs.
(function governanceBoot(core, mcp, use, host) {
  'use strict';
  const base = core(host);
  if (base) use(base, mcp(base));
})
