# Context Indexer

Developer tool for navigating symbols, requirements, and tests.

## Usage

```bash
uv run python devtools/context-indexer/main.py --find-symbol Price
uv run python devtools/context-indexer/main.py --find-references ProbabilityScaled
uv run python devtools/context-indexer/main.py --get-related-tests IPC-001
```

## Design

Uses exact code search and structural navigation first.
Semantic embeddings are secondary.

## NOT part of the research runtime.
