//! research-tool — CLI for querying GPT-5.5:online via OpenRouter
//!
//! A lightweight research assistant that queries OpenAI's GPT-5.5 model
//! through OpenRouter with web search and chain-of-thought reasoning.
//!
//! ## Quick start
//!
//!   export OPENROUTER_API_KEY="sk-or-v1-..."
//!   research-tool "What are the latest OpenRouter API features in 2026?"
//!
//! ## Features
//!
//! - **Web search**: The default model (openai/gpt-5.5:online) has live web
//!   access and can cite current sources, prices, news, and documentation.
//! - **Chain-of-thought reasoning**: Adjustable reasoning effort (low → xhigh)
//!   for simple lookups vs deep analysis.
//! - **Pipe-friendly**: Response goes to stdout, metadata (reasoning, token
//!   stats) goes to stderr. Pipe output to files or other tools cleanly.
//! - **Custom system prompts**: Override the default research assistant persona
//!   for domain-specific queries.
//!
//! ## Environment variables
//!
//! - OPENROUTER_API_KEY (required): Your OpenRouter API key.
//! - RESEARCH_MODEL: Override default model (default: openai/gpt-5.5:online).
//! - RESEARCH_EFFORT: Override default reasoning effort (default: high).
//!
//! ## Examples
//!
//!   # Basic research query
//!   research-tool "What is the current population of Tokyo?"
//!
//!   # Multi-word queries (no quotes needed)
//!   research-tool what are the best rust async patterns
//!
//!   # Higher reasoning effort for complex analysis
//!   research-tool --effort xhigh "Compare tradeoffs between Opus 4.6 and gpt-5.3-codex for programming"
//!
//!   # Custom system prompt for domain expertise
//!   research-tool --system "You are a senior infrastructure engineer" "Best practices for zero-downtime Kubernetes deployments"
//!
//!   # Pipe from stdin (useful for long/multiline queries)
//!   echo "Explain the OpenRouter model routing architecture" | research-tool --stdin
//!
//!   # Save output to file
//!   research-tool "Summarize recent changes to the GitHub Actions runner architecture" > research-output.md
//!
//!   # Use a different model (e.g., without web search)
//!   research-tool --model openai/gpt-5.5 "Explain the React Server Components architecture"
//!
//!   # Longer timeout for complex web research
//!   research-tool --timeout 180 "What are the most popular Rust web frameworks in 2026?"

use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};

/// Query GPT-5.5:online for research via OpenRouter.
///
/// Sends your question to OpenAI's GPT-5.5 model through OpenRouter with
/// web search enabled and chain-of-thought reasoning. The model can access
/// live web data, cite sources, and perform deep analysis.
///
/// Response text is printed to stdout. Reasoning traces, progress indicators,
/// and token usage stats are printed to stderr (pipe-friendly).
///
/// Requires OPENROUTER_API_KEY environment variable to be set.
///
/// EXAMPLES:
///
///   # Simple research query
///   research-tool "What is the current population of Tokyo?"
///
///   # Deep analysis with maximum reasoning
///   research-tool --effort xhigh "Compare Next.js vs Remix for full-stack web applications"
///
///   # Custom persona for domain-specific research
///   research-tool -s "You are a Rust systems programmer" "Best async patterns for WebSocket servers"
///
///   # Pipe from stdin
///   cat question.txt | research-tool --stdin
///
///   # Save output to a file (only response, no metadata)
///   research-tool "Summarize recent changes to the OpenAI API in 2026" > summary.md
#[derive(Parser)]
#[command(
    name = "research-tool",
    version,
    after_help = "OUTPUT:\n  \
        stdout: Model response text (pipe-friendly)\n  \
        stderr: Progress indicator, reasoning trace, token usage stats\n\n\
        COST:\n  \
        ~$0.01-0.05 per query depending on response length and reasoning effort.\n  \
        Token usage is printed to stderr after each query.\n\n\
        AUTHENTICATION:\n  \
        Set OPENROUTER_API_KEY in your environment or .env file.\n  \
        Get a key at https://openrouter.ai/keys"
)]
struct Args {
    /// The question or research query (multiple words joined automatically).
    /// Wrap in quotes for clarity, or just type naturally:
    ///   research-tool "What is the population of Tokyo?"
    ///   research-tool what is the population of tokyo
    #[arg(trailing_var_arg = true)]
    query: Vec<String>,

    /// Read the query from stdin instead of command-line arguments.
    /// Useful for long or multiline queries:
    ///   echo "my question" | research-tool --stdin
    ///   cat prompt.txt | research-tool --stdin
    #[arg(long)]
    stdin: bool,

    /// AI model to use for the query.
    /// The ":online" suffix enables live web search (recommended).
    /// Without ":online", the model has no web access (training data only).
    ///   openai/gpt-5.5:online  — GPT-5.5 + web search (default)
    ///   openai/gpt-5.5         — GPT-5.5 without web search
    ///   openai/gpt-5.5-pro:online — GPT-5.5 Pro + web search for heavier research
    ///   anthropic/claude-opus-4-6 — Claude Opus (no web search)
    #[arg(
        long,
        short,
        default_value = "openai/gpt-5.5:online",
        env = "RESEARCH_MODEL",
        verbatim_doc_comment
    )]
    model: String,

    /// Reasoning effort level — controls how much the model "thinks" before
    /// answering. Higher effort = better analysis but slower and more tokens.
    ///   low    — Quick lookup, minimal reasoning (~1-5s, default)
    ///   medium — Standard analysis (~5-15s)
    ///   high   — Deep analysis with careful reasoning (~15-60s)
    ///   xhigh  — Maximum reasoning effort (~30-120s, best for complex questions)
    #[arg(
        long,
        short,
        default_value = "low",
        env = "RESEARCH_EFFORT",
        verbatim_doc_comment
    )]
    effort: String,

    /// Override the system prompt (persona/instructions for the model).
    /// Default: general research assistant that cites sources.
    /// Use this to specialize for a domain:
    ///   -s "You are a senior infrastructure engineer"
    ///   -s "You are a Rust systems programmer"
    #[arg(long, short, verbatim_doc_comment)]
    system: Option<String>,

    /// Maximum number of tokens in the response. Higher values allow longer
    /// answers but cost more. Most research answers fit in 2000-4000 tokens.
    #[arg(long, default_value = "12800")]
    max_tokens: u32,

    /// Request timeout in seconds. No timeout by default — queries complete
    /// when the model finishes. Set this only if you need a hard upper bound.
    #[arg(long)]
    timeout: Option<u64>,

    /// OpenRouter API key (reads from OPENROUTER_API_KEY env var by default).
    /// Only needed as a flag to override the env var.
    #[arg(long, env = "OPENROUTER_API_KEY", hide = true, hide_env = true)]
    api_key: Option<String>,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning: Option<Reasoning>,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct Reasoning {
    effort: String,
}

#[derive(Deserialize, Debug)]
struct ChatResponse {
    choices: Option<Vec<Choice>>,
    usage: Option<Usage>,
    error: Option<ApiError>,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: Option<MessageContent>,
}

#[derive(Deserialize, Debug)]
struct MessageContent {
    content: Option<String>,
    reasoning: Option<String>,
    #[serde(default)]
    reasoning_content: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Deserialize, Debug)]
struct ApiError {
    message: String,
    #[allow(dead_code)]
    code: Option<serde_json::Value>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file for OPENROUTER_API_KEY (checks current dir, then home)
    for env_path in &[
        std::path::PathBuf::from(".env"),
        dirs::home_dir().unwrap_or_default().join(".env"),
    ] {
        if env_path.exists() {
            dotenvy::from_path(env_path).ok();
            break;
        }
    }

    let args = Args::parse();

    let api_key = args.api_key.unwrap_or_else(|| {
        eprintln!("❌ No API key found.\n\nSet OPENROUTER_API_KEY in your environment:\n  export OPENROUTER_API_KEY=\"sk-or-v1-...\"\n\nGet a key at https://openrouter.ai/keys");
        std::process::exit(1);
    });

    // Build query from args or stdin
    let query = if args.stdin {
        let mut buf = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)?;
        buf.trim().to_string()
    } else if args.query.is_empty() {
        anyhow::bail!(
            "No query provided.\n\n\
             Usage: research-tool \"your question here\"\n\
             Help:  research-tool --help"
        );
    } else {
        args.query.join(" ")
    };

    eprintln!(
        "🔍 Researching with {} (effort: {})...",
        args.model, args.effort
    );

    let mut client_builder = reqwest::Client::builder();
    if let Some(timeout) = args.timeout {
        client_builder = client_builder.timeout(Duration::from_secs(timeout));
    }
    let client = client_builder.build()?;

    let mut messages = Vec::new();

    let system_prompt = args.system.unwrap_or_else(|| {
        "You are a research assistant. Provide detailed, accurate answers with \
         sources and citations where possible. Focus on factual, verifiable \
         information. When citing web sources, include URLs."
            .into()
    });

    messages.push(ChatMessage {
        role: "system".into(),
        content: system_prompt,
    });

    messages.push(ChatMessage {
        role: "user".into(),
        content: query,
    });

    let body = ChatRequest {
        model: args.model.clone(),
        messages,
        max_tokens: Some(args.max_tokens),
        reasoning: Some(Reasoning {
            effort: args.effort.clone(),
        }),
    };

    // Elapsed timer — updates every second so agents and users know the process is alive
    let start = std::time::Instant::now();
    use std::io::IsTerminal;
    let is_tty = std::io::stderr().is_terminal();
    let timer_start = start;
    let timer_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        interval.tick().await; // skip immediate first tick
        loop {
            interval.tick().await;
            let elapsed = timer_start.elapsed().as_secs();
            if is_tty {
                eprint!("\r⏳ {}s ", elapsed);
            } else {
                eprintln!("⏳ {}s", elapsed);
            }
        }
    });

    let resp = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header(
            "HTTP-Referer",
            "https://github.com/aaronn/openclaw-search-tool",
        )
        .header("X-Title", "OpenClaw Research Tool")
        .json(&body)
        .send()
        .await
        .context("❌ Connection to OpenRouter failed — check your network and retry?")?;

    eprintln!("✅ Connected — waiting for response...");

    let status = resp.status();
    let text = resp
        .text()
        .await
        .context("❌ Connection to OpenRouter lost while waiting for response. Retry?")?;

    timer_handle.abort();

    if !status.is_success() {
        if status.as_u16() == 401 {
            anyhow::bail!(
                "Authentication failed (401). Check your OPENROUTER_API_KEY.\n\
                 Get a key at https://openrouter.ai/keys"
            );
        } else if status.as_u16() == 402 {
            anyhow::bail!(
                "Insufficient credits (402). Add credits at https://openrouter.ai/credits"
            );
        } else if status.as_u16() == 429 {
            anyhow::bail!("Rate limited (429). Wait a moment and try again.");
        } else {
            anyhow::bail!("API error ({}): {}", status, text);
        }
    }

    let response: ChatResponse = serde_json::from_str(&text).context(format!(
        "Failed to parse API response: {}",
        &text[..200.min(text.len())]
    ))?;

    if let Some(error) = response.error {
        anyhow::bail!("API error: {}", error.message);
    }

    if let Some(choices) = &response.choices {
        if let Some(choice) = choices.first() {
            if let Some(msg) = &choice.message {
                // Print reasoning trace to stderr (if model returned one)
                let reasoning = msg.reasoning.as_ref().or(msg.reasoning_content.as_ref());
                if let Some(r) = reasoning {
                    if !r.is_empty() {
                        eprintln!("\n💭 Reasoning:\n{}\n---", r);
                    }
                }

                // Print response to stdout (pipe-friendly)
                if let Some(content) = &msg.content {
                    println!("{}", content);
                } else {
                    eprintln!("⚠️ No content in response");
                }
            }
        }
    } else {
        eprintln!("⚠️ No choices in response");
    }

    // Print usage stats to stderr
    let elapsed = start.elapsed().as_secs();
    if let Some(usage) = &response.usage {
        eprintln!(
            "\n📊 Tokens: {} prompt + {} completion = {} total | ⏱ {}s",
            usage.prompt_tokens, usage.completion_tokens, usage.total_tokens, elapsed
        );
    } else {
        eprintln!("\n⏱ {}s", elapsed);
    }

    // Force exit — reqwest's connection pool keeps tokio alive otherwise
    std::process::exit(0);
}
