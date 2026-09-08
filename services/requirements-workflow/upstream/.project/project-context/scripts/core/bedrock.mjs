// AWS Bedrock client: the only place that talks to a model provider.
//
// Auth is a Bedrock API key (bearer token) in AWS_BEARER_TOKEN_BEDROCK, read by
// the AWS SDK automatically. Rerank goes through bedrock-runtime InvokeModel (the
// agent-runtime Rerank op rejects the bearer token), not the agent-runtime API.

import {
  BedrockRuntimeClient,
  InvokeModelCommand,
  ConverseCommand,
} from '@aws-sdk/client-bedrock-runtime';

import { warn } from './log.mjs';
import { mapLimit, createLimiter } from './concurrency.mjs';

const decoder = new TextDecoder();

function decodeJson(bytes) {
  return JSON.parse(decoder.decode(bytes));
}

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

/** Send a Bedrock command with exponential backoff + jitter on throttling. */
async function sendWithRetry(client, command, { maxRetries = 10, baseDelay = 500 } = {}) {
  let attempt = 0;
  for (;;) {
    try {
      return await client.send(command);
    } catch (err) {
      if (!isRetryable(err) || attempt >= maxRetries) throw err;
      const delay = Math.min(baseDelay * 2 ** attempt, 30_000) + Math.random() * 500;
      attempt++;
      warn(`Bedrock ${err.name ?? 'error'} - retry ${attempt}/${maxRetries} in ${Math.round(delay)}ms`);
      await sleep(delay);
    }
  }
}

/**
 * @param {object} config loaded config (see config.mjs)
 * @returns {{ embed, converse, rerank, check, embedModel, chunkModel, rerankModel }}
 */
export function createBedrock(config) {
  const { region, embedModel, embedDim, chunkModel, rerankModel } = config.bedrock;

  if (!process.env.AWS_BEARER_TOKEN_BEDROCK && !process.env.AWS_ACCESS_KEY_ID) {
    throw new Error(
      'No Bedrock credentials. Set AWS_BEARER_TOKEN_BEDROCK (API key) or an AWS key pair in .env.',
    );
  }

  // Some third-party models occasionally hang with no response and no thrown
  // error (observed with Gemma 3 27B in eu-west-1). Without a request timeout
  // such a call blocks its slot forever; sendWithRetry only retries on
  // exceptions, so a hang needs to become an exception first.
  const client = new BedrockRuntimeClient({
    region,
    maxAttempts: 1,
    requestHandler: { requestTimeout: 90_000, connectionTimeout: 10_000 },
  });
  const embedConcurrency = config.bedrock.embedConcurrency ?? 16;
  const limit = createLimiter(config.bedrock.maxConcurrency ?? 8);
  const send = (command) => limit(() => sendWithRetry(client, command));

  async function embedOne(text) {
    const body = JSON.stringify({ inputText: text, dimensions: embedDim, normalize: true });
    const res = await send(
      new InvokeModelCommand({
        modelId: embedModel,
        contentType: 'application/json',
        accept: 'application/json',
        body,
      }),
    );
    const json = decodeJson(res.body);
    return json.embedding;
  }

  /** Embed a batch of texts. Titan V2 embeds one text per call; we fan out. */
  async function embed(texts, { concurrency = embedConcurrency } = {}) {
    return mapLimit(texts, concurrency, embedOne);
  }

  /** Single-turn chat via the Converse API. Returns the assistant text. */
  async function converse(system, user, { temperature = 0, maxTokens = 4096, modelId = chunkModel } = {}) {
    const res = await send(
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

  /**
   * Rerank documents against a query. Returns [{ index, score }] sorted desc.
   * Cohere and Amazon rerank share the response shape; only the request differs.
   */
  async function rerank(query, documents, { topN = documents.length } = {}) {
    if (documents.length === 0) return [];
    const isCohere = rerankModel.startsWith('cohere.');
    const body = isCohere
      ? JSON.stringify({ query, documents, top_n: topN, api_version: 2 })
      : JSON.stringify({ query, documents, top_n: topN });

    const res = await send(
      new InvokeModelCommand({
        modelId: rerankModel,
        contentType: 'application/json',
        accept: 'application/json',
        body,
      }),
    );
    const json = decodeJson(res.body);
    return (json.results ?? [])
      .map((r) => ({ index: r.index, score: r.relevance_score }))
      .sort((a, b) => b.score - a.score);
  }

  /** Cheap liveness/creds check: embed a token and confirm the dimension. */
  async function check() {
    const [v] = await embed(['ping']);
    if (!Array.isArray(v) || v.length !== embedDim) {
      throw new Error(`embed returned ${v?.length} dims, expected ${embedDim}`);
    }
    return { region, embedModel, embedDim: v.length };
  }

  return { embed, converse, rerank, check, embedModel, chunkModel, rerankModel };
}
