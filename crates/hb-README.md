# hb

## The Host Boundary Stack

**Atmospheric infrastructure for Cloudflare Workers.**

hb is a cohesive collection of Rust crates designed to build correct, high-performance, and observable distributed systems on the Cloudflare edge.

It is not a monolithic framework. It is a set of sharp, single-purpose tools that share a common DNA: Type safety, zero-cost abstractions, and deep observability.

> "Fast, invisible, essential. Like a hot breeze in a cold landscape."

## The Stack

Each crate is designed to be used independently, but they sing when used together.

| Crate | Role | Type | Description |
|-------|------|------|-------------|
| **hb-d1c** | Data | CLI | Type-safe SQL generator for D1. Compiles your `.sql` files into checked Rust functions. Supports atomic batching and multi-file modules. |
| **hb-auth** | Identity | Library | Drop-in Cloudflare Access JWT validation with a strongly-typed permission DSL. |
| **hb-sync** | State | Library | Synchronization primitives (Mutex, RWLock) optimized for Durable Object storage. |
| **hb-flags** | Control | Library | **Coming Soon**. Typed feature flags backed by KV with edge-local caching. |
| **hb-secrets** | Security | Library | **Coming Soon**. Compile-time validated secret loading to prevent runtime configuration panics. |

## Philosophy

We believe the edge requires a different set of tools than the server.

- **Correctness at Scale:** Distributed systems are hard. hb uses Rust's type system to make invalid states unrepresentable (e.g., `hb-d1c` refuses to compile if your SQL binds an integer to a text column).
- **Observability First:** You cannot fix what you cannot see. Turn on Cloudflare Workers Observability (built-in traces/logs) before anything else.
- **Native Primitives:** We do not fight the platform. We embrace Durable Objects, Queues, and D1 as they are, rather than trying to abstract them into generic "serverless" concepts.

## Installation

Add the components you need to your `Cargo.toml`:

```toml
[dependencies]
hb-auth = "0.1"
```

---

### A SaavyLab Production

Built with intention by [SaavyLab](https://hb.saavylab.dev).
