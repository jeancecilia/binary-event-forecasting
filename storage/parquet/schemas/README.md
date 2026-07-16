# Parquet Schemas

## market-events

Stores raw market events for replay and analysis.

```
data/traces/
└── source=<source>/
    └── year=<YYYY>/
        └── month=<MM>/
            └── market_id=<market-id>/
                └── part-00001.parquet
```

Columns:
- `event_id` (string)
- `market_id` (string)
- `event_type` (string: trade, quote_update, order_book_delta, order_book_snapshot, feed_status_change, settlement, correction)
- `source_timestamp` (timestamp[ms])
- `logical_timestamp` (int64)
- `source_sequence` (int64, nullable)
- `payload` (string, JSON)
- `source_valid_at` (timestamp[ms])
- `first_observed_at` (timestamp[ms])

## market-snapshots

Periodic snapshots of full order books.

Columns:
- `snapshot_id` (string)
- `market_id` (string)
- `snapshot_version` (int64)
- `logical_timestamp` (int64)
- `source_timestamp` (timestamp[ms])
- `feed_status` (string)
- `bids` (string, JSON array of [price, quantity])
- `asks` (string, JSON array of [price, quantity])
- `snapshot_hash` (string, SHA-256)

## inference-features

Feature vectors for model inference.

Columns:
- `feature_id` (string)
- `experiment_id` (string)
- `market_id` (string)
- `feature_vector` (list<float>)
- `generated_at` (timestamp[ms])
- `feature_version` (string)
