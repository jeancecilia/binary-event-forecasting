# ADR-012: Agent Context Retrieval Strategy

**Status:** Accepted  
**Date:** 2026-07-15  
**Decision:** Agents navigate modules using structural search and a repository map, not by ingesting the entire codebase.

## Context

LLM context windows are limited. Ingesting the full repository into context is wasteful and leads to hallucination. Agents need targeted information retrieval.

## Decision

- Root `AGENTS.md` provides mandatory reading order and navigation rules
- `docs/generated/repository-map.md` is auto-generated from code
- `devtools/context-indexer/` provides symbol search, requirement lookup, and test mapping
- Module READMEs summarize responsibility, public API, requirements, and tests

## Consequences

- Agents can find relevant code without reading everything
- Repository map must be regenerated after structural changes
- Module READMEs must be maintained
- Exact code search is primary; semantic embeddings are secondary
