---
name: research-tool
description: Search the web using LLMs via OpenRouter. Use for current web data, API docs, market research, news, fact-checking, or any question that benefits from live internet access and reasoning.
metadata: {"openclaw": {"emoji": "🔍", "requires": {"bins": ["research-tool"], "env": ["OPENROUTER_API_KEY"]}, "primaryEnv": "OPENROUTER_API_KEY", "install": [{"kind": "download", "url": "https://github.com/aaronn/openclaw-search-tool", "note": "cargo install openclaw-search-tool"}]}}
---

# OpenClaw Search Tool

A CLI for OpenClaw agents to search the web using LLMs via OpenRouter. The `:online` model suffix gives any model **live web access** — it searches the web, reads pages, cites URLs, and synthesizes an answer.

**Search in natural language.** Write your query the way you'd ask a person, not as keywords.

## Install

```bash
cargo install openclaw-search-tool
```

Requires `OPENROUTER_API_KEY` env var. Get a key at https://openrouter.ai/keys

## Quick start

```bash
research-tool "What are the x.com API rate limits?"
research-tool "How do I set reasoning effort parameters on OpenRouter?"
```

## Flags

### `--effort`, `-e` (default: `low`)

Controls how much the model reasons before answering. Higher effort means better analysis but slower and more tokens.

```bash
research-tool --effort low "What year was Rust 1.0 released?"
research-tool --effort medium "Explain how OpenRouter routes requests to different model providers"
research-tool --effort high "Compare tradeoffs between Opus 4.6 and gpt-5.3-codex for programming"
research-tool --effort xhigh "Deep analysis of React Server Components vs traditional SSR approaches"
```

| Level | Speed | When to use |
|-------|-------|-------------|
| `low` | ~1-5s | Quick fact lookups, simple questions |
| `medium` | ~5-15s | Standard research, moderate analysis |
| `high` | ~15-60s | Deep analysis with careful reasoning |
| `xhigh` | ~30-120s+ | Maximum reasoning, complex multi-source synthesis |

Can also be set via env var `RESEARCH_EFFORT`.

### `--model`, `-m` (default: `openai/gpt-5.2:online`)

Which model to use. The `:online` suffix enables live web search and works with **any model on OpenRouter**.

```bash
# Default: GPT-5.2 with web search
research-tool "current weather in San Francisco"

# Claude with web search
research-tool -m "anthropic/claude-sonnet-4-20250514:online" "Summarize recent changes to the OpenAI API"

# GPT-5.2 without web search (training data only)
research-tool -m "openai/gpt-5.2" "Explain the React Server Components architecture"

# Any OpenRouter model
research-tool -m "google/gemini-2.5-pro:online" "Compare React vs Svelte in 2026"
```

Can also be set via env var `RESEARCH_MODEL`.

### `--system`, `-s`

Override the system prompt to give the model a specific persona or instructions.

```bash
research-tool -s "You are a senior infrastructure engineer" "Best practices for zero-downtime Kubernetes deployments"
research-tool -s "You are a Rust systems programmer" "Best async patterns for WebSocket servers"
```

### `--stdin`

Read the query from stdin. Useful for long or multiline queries.

```bash
echo "Explain the OpenRouter model routing architecture" | research-tool --stdin
cat detailed-prompt.txt | research-tool --stdin
```

### `--max-tokens` (default: `12800`)

Maximum tokens in the response.

### `--timeout` (default: `600`)

Request timeout in seconds. Default is 10 minutes for deep web searches.

## Output format

- **stdout**: Response text only (markdown with citations) — pipe-friendly
- **stderr**: Progress status, reasoning traces, and token usage

```
🔍 Researching with openai/gpt-5.2:online (effort: high)...
✅ Connected — waiting for response...

[response text on stdout]

📊 Tokens: 4470 prompt + 184 completion = 4654 total | ⏱ 5s
```

## Status indicators

- `🔍 Researching...` — request sent to OpenRouter
- `✅ Connected — waiting for response...` — server accepted the request, model is searching/thinking
- `⏳ 15s... ⏳ 30s...` — elapsed time ticks (only in interactive terminals, not in agent exec)
- `❌ Connection to OpenRouter failed` — couldn't reach OpenRouter (network issue)
- `❌ Connection to OpenRouter lost` — connection dropped while waiting. Retry?

## Tips for better results

- **Write in natural language.** "What are the best practices for Rust error handling and when should you use anyhow vs thiserror?" works better than keyword-style queries.
- **Provide maximum context.** The model starts from zero. Include background, what you already know, and all related sub-questions. Detailed prompts massively outperform vague ones.
- **Use effort levels appropriately.** `low` for quick facts, `high` for real research, `xhigh` only for complex multi-source analysis.
- **Use `-s` for domain expertise.** A specific persona produces noticeably better domain-specific answers.

## Cost

~$0.01–0.05 per query. Token usage is printed to stderr after each query.
