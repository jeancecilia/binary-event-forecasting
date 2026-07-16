//! SQLite journal database operations.

use rusqlite::Connection;

/// Open the journal database with recommended settings.
pub fn open_journal(path: &str) -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open(path)?;

    // Recommended configuration for crash-safe operation
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = FULL;
         PRAGMA foreign_keys = ON;",
    )?;

    // Create core tables
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS journal_records (
            record_id TEXT PRIMARY KEY,
            entity_id TEXT NOT NULL,
            lifecycle_state TEXT NOT NULL,
            transition_id TEXT NOT NULL,
            logical_timestamp INTEGER NOT NULL,
            canonical_payload_hash TEXT NOT NULL,
            previous_record_hash TEXT NOT NULL,
            checksum TEXT NOT NULL,
            created_at_runtime TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS message_receipts (
            message_id TEXT PRIMARY KEY,
            receipt_status TEXT NOT NULL,
            received_at TEXT NOT NULL,
            payload_hash TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS message_dispositions (
            message_id TEXT NOT NULL,
            disposition_status TEXT NOT NULL,
            transition_id TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            PRIMARY KEY (message_id, transition_id)
        );

        CREATE TABLE IF NOT EXISTS transition_plans (
            transition_id TEXT PRIMARY KEY,
            entity_id TEXT NOT NULL,
            planned_at TEXT NOT NULL,
            payload_hash TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS transition_commits (
            transition_id TEXT PRIMARY KEY,
            entity_id TEXT NOT NULL,
            committed_at TEXT NOT NULL,
            final_hash TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS sender_sequences (
            sender_instance_id TEXT NOT NULL,
            sender_sequence INTEGER NOT NULL,
            last_seen_at TEXT NOT NULL,
            PRIMARY KEY (sender_instance_id, sender_sequence)
        );

        CREATE TABLE IF NOT EXISTS processed_messages (
            message_id TEXT PRIMARY KEY,
            processed_at TEXT NOT NULL,
            terminal_disposition TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS ledger_checkpoints (
            checkpoint_id TEXT PRIMARY KEY,
            ledger_version INTEGER NOT NULL,
            free_cash TEXT NOT NULL,
            reserved_cash TEXT NOT NULL,
            total_cash TEXT NOT NULL,
            created_at TEXT NOT NULL,
            state_hash TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_journal_entity
            ON journal_records(entity_id);
        CREATE INDEX IF NOT EXISTS idx_journal_transition
            ON journal_records(transition_id);
        CREATE INDEX IF NOT EXISTS idx_journal_timestamp
            ON journal_records(logical_timestamp);",
    )?;

    Ok(conn)
}
