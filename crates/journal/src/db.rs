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
            payload TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS transition_applications (
            transition_id TEXT PRIMARY KEY,
            entity_id TEXT NOT NULL,
            applied_at TEXT NOT NULL,
            payload TEXT NOT NULL
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
    let existing_hash: Option<String> = tx
        .query_row(
            "SELECT payload_hash FROM message_receipts WHERE message_id = ?1",
            [message_id],
            |row| row.get(0),
        )
        .optional()?;

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

pub fn commit_transition_plan(
    conn: &mut Connection,
    transition_id: &str,
    entity_id: &str,
    planned_at: &str,
    payload: &str, // Changed from payload_hash
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO transition_plans (transition_id, entity_id, planned_at, payload) 
         VALUES (?1, ?2, ?3, ?4)",
        [transition_id, entity_id, planned_at, payload],
    )?;
    Ok(())
}

pub fn commit_transition_application(
    conn: &mut Connection,
    transition_id: &str,
    entity_id: &str,
    applied_at: &str,
    payload: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO transition_applications (transition_id, entity_id, applied_at, payload) 
         VALUES (?1, ?2, ?3, ?4)",
        [transition_id, entity_id, applied_at, payload],
    )?;
    Ok(())
}

pub fn commit_terminal_disposition(
    conn: &mut Connection,
    transition_id: &str,
    entity_id: &str,
    committed_at: &str,
    final_hash: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO transition_commits (transition_id, entity_id, committed_at, final_hash) 
         VALUES (?1, ?2, ?3, ?4)",
        [transition_id, entity_id, committed_at, final_hash],
    )?;
    Ok(())
}

pub fn save_ledger_state(
    conn: &mut Connection,
    checkpoint_id: &str,
    ledger_version: u64,
    free_cash: &str,     // Kept for schema backwards compatibility or quick querying
    reserved_cash: &str, // Kept for schema backwards compatibility
    total_cash: &str,    // Will store the full JSON payload here to avoid schema changes
    created_at: &str,
    state_hash: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO ledger_checkpoints (checkpoint_id, ledger_version, free_cash, reserved_cash, total_cash, created_at, state_hash) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        [
            checkpoint_id,
            &ledger_version.to_string(),
            free_cash,
            reserved_cash,
            total_cash, // Contains full JSON string of the Ledger
            created_at,
            state_hash,
        ],
    )?;
    Ok(())
}

pub fn load_ledger_state(
    conn: &Connection,
) -> Result<Option<(u64, String, String, String)>, rusqlite::Error> {
    // Returns (version, free, reserved, total) where total holds the JSON string
    let mut stmt = conn.prepare(
        "SELECT ledger_version, free_cash, reserved_cash, total_cash 
         FROM ledger_checkpoints 
         ORDER BY ledger_version DESC LIMIT 1",
    )?;
    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        let version: u64 = row.get(0)?;
        let free: String = row.get(1)?;
        let res: String = row.get(2)?;
        let total: String = row.get(3)?; // the full JSON payload
        Ok(Some((version, free, res, total)))
    } else {
        Ok(None)
    }
}

pub fn load_applied_transitions(
    conn: &Connection,
) -> Result<std::collections::BTreeSet<String>, rusqlite::Error> {
    // We now load applied transitions from transition_applications (applied but maybe not committed)
    // combined with transition_commits (terminal).
    let mut stmt = conn.prepare("SELECT transition_id FROM transition_applications UNION SELECT transition_id FROM transition_commits")?;
    let mut rows = stmt.query([])?;
    let mut applied = std::collections::BTreeSet::new();
    while let Some(row) = rows.next()? {
        applied.insert(row.get(0)?);
    }
    Ok(applied)
}

pub fn load_pending_transitions(
    conn: &Connection,
) -> Result<Vec<(String, String, String)>, rusqlite::Error> {
    // Returns (transition_id, entity_id, payload) of plans that have no commit.
    let mut stmt = conn.prepare(
        "SELECT p.transition_id, p.entity_id, p.payload 
         FROM transition_plans p
         LEFT JOIN transition_commits c ON p.transition_id = c.transition_id
         WHERE c.transition_id IS NULL 
         ORDER BY p.planned_at ASC",
    )?;
    let mut rows = stmt.query([])?;
    let mut pending = Vec::new();
    while let Some(row) = rows.next()? {
        pending.push((row.get(0)?, row.get(1)?, row.get(2)?));
    }
    Ok(pending)
}
