# OpenClaw Research Tool

A tool for [OpenClaw](https://openclaw.ai) agents to search the web using LLMs via [OpenRouter](https://openrouter.ai). Ask questions in natural language, get cited answers with sources.

```bash
research-tool "How do I set reasoning effort parameters on OpenRouter?"
```

Built on [OpenRouter](https://openrouter.ai), which gives any model live web search via the `:online` suffix. The default model is `openai/gpt-5.2:online`, but you can use any model OpenRouter supports.

## Install

```bash
cargo install --path .
```

## Setup

Get an API key from [OpenRouter](https://openrouter.ai/keys) and set it in your environment:

```bash
export OPENROUTER_API_KEY="sk-or-v1-..."
```

Or add it to a `.env` file in your working directory.

## Usage

Search in natural language — write your query the way you'd ask a person:

```bash
# Simple question
research-tool "What are the x.com API rate limits?"

# Deep analysis
research-tool --effort xhigh "Compare tradeoffs between Opus 4.6 and gpt-5.3-codex for programming"

# Quick fact check
research-tool --effort low "What year was Rust 1.0 released?"

# Custom persona
research-tool -s "You are a senior infrastructure engineer" "Best practices for zero-downtime Kubernetes deployments"

# Use a different model
research-tool -m "anthropic/claude-sonnet-4-20250514:online" "Summarize recent changes to the OpenAI API"

# Pipe from stdin
cat question.txt | research-tool --stdin

# Save output (response goes to stdout, metadata to stderr)
research-tool "Explain the React Server Components architecture" > output.md
```

## Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--model` | `-m` | `openai/gpt-5.2:online` | Model to use. Defaults to GPT-5.2 — great for cited answers and docs. Append `:online` to any model for web search. |
| `--effort` | `-e` | `low` | Reasoning effort: `low`, `medium`, `high`, `xhigh` |
| `--system` | `-s` | Research assistant | Custom system prompt / persona |
| `--max-tokens` | | `12800` | Max response tokens |
| `--timeout` | | `600` | Request timeout in seconds |
| `--stdin` | | | Read query from stdin |

## How it works

1. Your query is sent to OpenRouter's chat completions API
2. The `:online` model variant enables live web search — the model browses the web, reads pages, and synthesizes an answer
3. Response text goes to **stdout** (pipe-friendly), reasoning traces and token stats go to **stderr**
4. Connection status is printed so you know if the search is still running or failed

## Tips

- **Write naturally.** "What are the best practices for Rust error handling?" works better than keyword-style queries.
- **Provide context.** The model starts from zero — the more detail you give, the better the answer. A 200-word question with background context will outperform a 5-word question.
- **Use effort levels.** `--effort low` for quick lookups (seconds), `--effort xhigh` for deep research (minutes).
- **Any model works with `:online`.** Try `anthropic/claude-opus-4-6:online` or `google/gemini-2.5-pro:online` for different perspectives.

## Output

```
🔍 Researching with openai/gpt-5.2:online (effort: high)...
✅ Connected — waiting for response...

[response text to stdout]

📊 Tokens: 4470 prompt + 184 completion = 4654 total | ⏱ 5s
```

## Cost

Roughly $0.01–0.05 per query depending on response length and reasoning effort. Token usage is printed after each query.

## License

MIT
