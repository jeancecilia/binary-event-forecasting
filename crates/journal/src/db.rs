//! SQLite journal database operations.

use rusqlite::{Connection, OptionalExtension};

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

pub fn process_forecast_receipt(
    conn: &mut Connection,
    message_id: &str,
    sender_instance_id: &str,
    sender_sequence: u64,
    payload_hash: &str,
    timestamp: &str,
) -> Result<protocol::enums::ReceiptStatus, rusqlite::Error> {
    let tx = conn.transaction()?;

    // 1. Check if message_id exists
    let existing_hash: Option<String> = tx.query_row(
        "SELECT payload_hash FROM message_receipts WHERE message_id = ?1",
        [message_id],
        |row| row.get(0),
    ).optional()?;

    if let Some(hash) = existing_hash {
        if hash == payload_hash {
            return Ok(protocol::enums::ReceiptStatus::DuplicateRetry);
        } else {
            return Ok(protocol::enums::ReceiptStatus::RejectedSchema);
        }
    }

    // 2. Check sender sequence regression
    let current_seq: Option<u64> = tx.query_row(
        "SELECT MAX(sender_sequence) FROM sender_sequences WHERE sender_instance_id = ?1",
        [sender_instance_id],
        |row| row.get::<_, Option<u64>>(0),
    )?;

    if let Some(seq) = current_seq {
        if sender_sequence <= seq {
            return Ok(protocol::enums::ReceiptStatus::ReplaySequenceViolation);
        }
    }

    // Insert receipt
    tx.execute(
        "INSERT INTO message_receipts (message_id, receipt_status, received_at, payload_hash) 
         VALUES (?1, ?2, ?3, ?4)",
        [message_id, "AcceptedQueued", timestamp, payload_hash],
    )?;

    // Update sender sequence
    tx.execute(
        "INSERT INTO sender_sequences (sender_instance_id, sender_sequence, last_seen_at) 
         VALUES (?1, ?2, ?3)",
        [sender_instance_id, &sender_sequence.to_string(), timestamp],
    )?;

    tx.commit()?;
    Ok(protocol::enums::ReceiptStatus::AcceptedQueued)
}
