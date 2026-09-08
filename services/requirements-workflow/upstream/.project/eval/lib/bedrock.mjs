// Minimal Amazon Bedrock client for the eval module.
//
// Self-contained: talks to Bedrock via the Converse API with exponential
// backoff on throttling. Auth is a Bedrock API key (bearer token) in
// AWS_BEARER_TOKEN_BEDROCK, read by the AWS SDK automatically. This module
// never reads or prints the .env file.
//
// Exposes two callables over the same model: `generate` (temperature > 0, so
// eval samples differ) and `judge` (temperature 0, for stable scoring).

import { BedrockRuntimeClient, ConverseCommand } from '@aws-sdk/client-bedrock-runtime';

const THROTTLE_NAMES = new Set([
  'ThrottlingException',
  'TooManyRequestsException',
  'ServiceUnavailableException',
  'ModelTimeoutException',
  'InternalServerException',
  'TimeoutError',
  'AbortError',
]);

function isRetryable(err) {
  if (!err) return false;
  if (THROTTLE_NAMES.has(err.name)) return true;
  if (err.name === 'ECONNRESET' || err.code === 'ECONNRESET') return true;
  const status = err.$metadata?.httpStatusCode;
  return status === 429 || status === 503 || status === 500;
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function sendWithRetry(client, command, { maxRetries = 8, baseDelay = 500 } = {}) {
  let attempt = 0;
  for (;;) {
    try {
      return await client.send(command);
    } catch (err) {
      if (!isRetryable(err) || attempt >= maxRetries) throw err;
      const delay = Math.min(baseDelay * 2 ** attempt, 30_000) + Math.random() * 500;
      attempt++;
      process.stderr.write(
        `Bedrock ${err.name ?? 'error'} - retry ${attempt}/${maxRetries} in ${Math.round(delay)}ms\n`,
      );
      await sleep(delay);
    }
  }
}

// Some models (e.g. Google Gemma 4) are served ONLY on the OpenAI-compatible
// `bedrock-mantle` endpoint, not the Converse API. Route those over HTTP.
function isMantleModel(modelId) {
  return typeof modelId === 'string' && modelId.startsWith('google.gemma-4');
}

async function fetchWithRetry(url, init, { maxRetries = 8, baseDelay = 500 } = {}) {
  let attempt = 0;
  for (;;) {
    let res;
    try {
      res = await fetch(url, init);
    } catch (err) {
      if (attempt >= maxRetries) throw err;
      const delay = Math.min(baseDelay * 2 ** attempt, 30_000) + Math.random() * 500;
      attempt++;
      process.stderr.write(`bedrock-mantle network error - retry ${attempt}/${maxRetries} in ${Math.round(delay)}ms\n`);
      await sleep(delay);
      continue;
    }
    if (res.ok) return res;
    const retryable = res.status === 429 || res.status === 503 || res.status === 500;
    if (!retryable || attempt >= maxRetries) {
      const text = await res.text().catch(() => '');
      throw new Error(`bedrock-mantle ${res.status}: ${text.slice(0, 600)}`);
    }
    const delay = Math.min(baseDelay * 2 ** attempt, 30_000) + Math.random() * 500;
    attempt++;
    process.stderr.write(`bedrock-mantle ${res.status} - retry ${attempt}/${maxRetries} in ${Math.round(delay)}ms\n`);
    await sleep(delay);
  }
}

/**
 * Create a Bedrock client with separate models for generation and judging.
 * @param {object} config full config ({ genModel, judgeModel, temperatures })
 * @returns {{ generate: fn, judge: fn, genModel: string, judgeModel: string, region: string }}
 */
export function createClient(config) {
  if (!process.env.AWS_BEARER_TOKEN_BEDROCK && !process.env.AWS_ACCESS_KEY_ID) {
    throw new Error(
      'No Bedrock credentials. Set AWS_BEARER_TOKEN_BEDROCK (API key) or an AWS key pair in .project/.env.',
    );
  }

  // Both roles share a region and one SDK client; they differ only by model id.
  const region = config.genModel.region;
  const client = new BedrockRuntimeClient({
    region,
    maxAttempts: 1,
    requestHandler: { requestTimeout: 180_000, connectionTimeout: 10_000 },
  });

  /** Single-turn OpenAI-compatible chat call on the bedrock-mantle endpoint. */
  async function mantleCall(modelId, maxTokens, system, user, temperature) {
    const token = process.env.AWS_BEARER_TOKEN_BEDROCK;
    if (!token) {
      throw new Error(
        `Model ${modelId} is served only on the bedrock-mantle endpoint, which needs an API key. Set AWS_BEARER_TOKEN_BEDROCK in .project/.env.`,
      );
    }
    const url = `https://bedrock-mantle.${region}.api.aws/openai/v1/chat/completions`;
    const messages = [];
    if (system) messages.push({ role: 'system', content: system });
    messages.push({ role: 'user', content: user });
    const res = await fetchWithRetry(url, {
      method: 'POST',
      headers: { Authorization: `Bearer ${token}`, 'Content-Type': 'application/json' },
      body: JSON.stringify({ model: modelId, messages, max_tokens: maxTokens, temperature }),
    });
    const json = await res.json();
    return (json.choices?.[0]?.message?.content ?? '').trim();
  }

  /** Single-turn Converse call on a given model at a given temperature. */
  async function converseCall(modelId, maxTokens, system, user, temperature) {
    const res = await sendWithRetry(
      client,
      new ConverseCommand({
        modelId,
        system: system ? [{ text: system }] : undefined,
        messages: [{ role: 'user', content: [{ text: user }] }],
        inferenceConfig: { temperature, maxTokens },
      }),
    );
    const parts = res.output?.message?.content ?? [];
    return parts.map((p) => p.text ?? '').join('').trim();
  }

  /** Route to the right transport for the model. */
  function call(modelId, maxTokens, system, user, temperature) {
    return isMantleModel(modelId)
      ? mantleCall(modelId, maxTokens, system, user, temperature)
      : converseCall(modelId, maxTokens, system, user, temperature);
  }

  const gen = config.genModel;
  const judge = config.judgeModel;
  return {
    generate: (system, user) => call(gen.id, gen.maxTokens, system, user, config.temperatures.generate),
    judge: (system, user) => call(judge.id, judge.maxTokens, system, user, config.temperatures.judge),
    genModel: gen.id,
    judgeModel: judge.id,
    region,
  };
}
