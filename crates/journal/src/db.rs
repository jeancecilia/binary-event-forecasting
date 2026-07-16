//! SQLite journal database operations.

use ledger::Ledger;
use rusqlite::{Connection, OptionalExtension, Transaction};

/// Errors raised while persisting or validating durable journal state.
#[derive(Debug, thiserror::Error)]
pub enum JournalDbError {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),

    #[error(transparent)]
    Serialization(#[from] serde_json::Error),

    #[error("Journal integrity violation: {0}")]
    Integrity(String),
}

/// Latest durable ledger checkpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerCheckpoint {
    pub ledger_version: u64,
    pub ledger_json: String,
    pub state_hash: String,
}

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

/// Persist an application record and the ledger state produced by it in one
/// SQLite transaction. Repeating the same operation is idempotent; conflicting
/// content for an existing transition or checkpoint fails closed.
#[allow(clippy::too_many_arguments)]
pub fn persist_transition_application_and_checkpoint(
    conn: &mut Connection,
    transition_id: &str,
    entity_id: &str,
    applied_at: &str,
    transition_payload: &str,
    checkpoint_id: &str,
    created_at: &str,
    ledger: &Ledger,
) -> Result<String, JournalDbError> {
    let ledger_json = protocol::canonical_json(ledger)?;
    let state_hash = protocol::canonical_hash(ledger)?;
    let ledger_version = ledger.version.to_string();
    let free_cash = ledger.free_cash.as_raw().to_string();
    let reserved_cash = ledger.reserved_cash.as_raw().to_string();

    let tx = conn.transaction()?;
    ensure_transition_application(
        &tx,
        transition_id,
        entity_id,
        applied_at,
        transition_payload,
    )?;
    ensure_ledger_checkpoint(
        &tx,
        checkpoint_id,
        &ledger_version,
        &free_cash,
        &reserved_cash,
        &ledger_json,
        created_at,
        &state_hash,
    )?;
    tx.commit()?;

    Ok(state_hash)
}

fn ensure_transition_application(
    tx: &Transaction<'_>,
    transition_id: &str,
    entity_id: &str,
    applied_at: &str,
    payload: &str,
) -> Result<(), JournalDbError> {
    let existing: Option<(String, String)> = tx
        .query_row(
            "SELECT entity_id, payload FROM transition_applications WHERE transition_id = ?1",
            [transition_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;

    if let Some((existing_entity_id, existing_payload)) = existing {
        if existing_entity_id != entity_id || existing_payload != payload {
            return Err(JournalDbError::Integrity(format!(
                "transition {transition_id} was already applied with different content"
            )));
        }
        return Ok(());
    }

    tx.execute(
        "INSERT INTO transition_applications (transition_id, entity_id, applied_at, payload)
         VALUES (?1, ?2, ?3, ?4)",
        [transition_id, entity_id, applied_at, payload],
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn ensure_ledger_checkpoint(
    tx: &Transaction<'_>,
    checkpoint_id: &str,
    ledger_version: &str,
    free_cash: &str,
    reserved_cash: &str,
    ledger_json: &str,
    created_at: &str,
    state_hash: &str,
) -> Result<(), JournalDbError> {
    let existing: Option<(u64, String, String)> = tx
        .query_row(
            "SELECT ledger_version, total_cash, state_hash
             FROM ledger_checkpoints WHERE checkpoint_id = ?1",
            [checkpoint_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?;

    if let Some((existing_version, existing_json, existing_hash)) = existing {
        let expected_version = ledger_version.parse::<u64>().map_err(|error| {
            JournalDbError::Integrity(format!("invalid ledger version: {error}"))
        })?;
        if existing_version != expected_version
            || existing_json != ledger_json
            || existing_hash != state_hash
        {
            return Err(JournalDbError::Integrity(format!(
                "checkpoint {checkpoint_id} already exists with different state"
            )));
        }
        return Ok(());
    }

    tx.execute(
        "INSERT INTO ledger_checkpoints
         (checkpoint_id, ledger_version, free_cash, reserved_cash, total_cash, created_at, state_hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        [
            checkpoint_id,
            ledger_version,
            free_cash,
            reserved_cash,
            ledger_json,
            created_at,
            state_hash,
        ],
    )?;
    Ok(())
}

/// Commit the terminal disposition exactly once. Identical retries succeed;
/// conflicting retries fail closed.
pub fn commit_terminal_disposition(
    conn: &mut Connection,
    transition_id: &str,
    entity_id: &str,
    committed_at: &str,
    final_hash: &str,
) -> Result<(), JournalDbError> {
    let existing: Option<(String, String)> = conn
        .query_row(
            "SELECT entity_id, final_hash FROM transition_commits WHERE transition_id = ?1",
            [transition_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;

    if let Some((existing_entity_id, existing_hash)) = existing {
        if existing_entity_id != entity_id || existing_hash != final_hash {
            return Err(JournalDbError::Integrity(format!(
                "transition {transition_id} was already committed with different content"
            )));
        }
        return Ok(());
    }

    conn.execute(
        "INSERT INTO transition_commits (transition_id, entity_id, committed_at, final_hash)
         VALUES (?1, ?2, ?3, ?4)",
        [transition_id, entity_id, committed_at, final_hash],
    )?;
    Ok(())
}

pub fn load_ledger_state(conn: &Connection) -> Result<Option<LedgerCheckpoint>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT ledger_version, total_cash, state_hash
         FROM ledger_checkpoints 
         ORDER BY ledger_version DESC, checkpoint_id DESC LIMIT 1",
    )?;
    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        Ok(Some(LedgerCheckpoint {
            ledger_version: row.get(0)?,
            ledger_json: row.get(1)?,
            state_hash: row.get(2)?,
        }))
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

#[cfg(test)]
mod tests {
    use super::*;
    use domain_types::Cash;
    use ledger::{Ledger, LedgerTransition};

    #[test]
    fn application_and_checkpoint_are_atomic_idempotent_and_hash_verified(
    ) -> Result<(), JournalDbError> {
        let mut conn = open_journal(":memory:")?;
        let mut ledger = Ledger::new(Cash::new(1_000));
        let transition = LedgerTransition {
            transition_id: "transition-1".to_string(),
            free_cash_delta: -100,
            reserved_cash_delta: 100,
            total_cash_delta: 0,
        };
        ledger
            .apply_transition(&transition)
            .map_err(JournalDbError::Integrity)?;
        let payload = serde_json::to_string(&transition)?;

        let state_hash = persist_transition_application_and_checkpoint(
            &mut conn,
            &transition.transition_id,
            "entity-1",
            "2026-01-01T00:00:00Z",
            &payload,
            "checkpoint-transition-1-1",
            "2026-01-01T00:00:00Z",
            &ledger,
        )?;

        // An identical retry is a no-op even when the retry timestamp differs.
        let retry_hash = persist_transition_application_and_checkpoint(
            &mut conn,
            &transition.transition_id,
            "entity-1",
            "2026-01-01T00:00:01Z",
            &payload,
            "checkpoint-transition-1-1",
            "2026-01-01T00:00:01Z",
            &ledger,
        )?;
        assert_eq!(state_hash, retry_hash);
        assert_eq!(state_hash, protocol::canonical_hash(&ledger)?);

        let checkpoint = load_ledger_state(&conn)?
            .ok_or_else(|| JournalDbError::Integrity("checkpoint was not persisted".to_string()))?;
        let restored = Ledger::restore_from_json(&checkpoint.ledger_json, &checkpoint.state_hash)
            .map_err(JournalDbError::Integrity)?;
        assert_eq!(restored.version, 1);
        assert_eq!(restored.free_cash.as_raw(), 900);
        assert_eq!(restored.reserved_cash.as_raw(), 100);

        let applications: u64 =
            conn.query_row("SELECT COUNT(*) FROM transition_applications", [], |row| {
                row.get(0)
            })?;
        let checkpoints: u64 =
            conn.query_row("SELECT COUNT(*) FROM ledger_checkpoints", [], |row| {
                row.get(0)
            })?;
        assert_eq!(applications, 1);
        assert_eq!(checkpoints, 1);
        Ok(())
    }

    #[test]
    fn conflicting_application_cannot_leave_a_checkpoint() -> Result<(), JournalDbError> {
        let mut conn = open_journal(":memory:")?;
        let ledger = Ledger::new(Cash::new(1_000));
        persist_transition_application_and_checkpoint(
            &mut conn,
            "transition-1",
            "entity-1",
            "2026-01-01T00:00:00Z",
            "payload-a",
            "checkpoint-a",
            "2026-01-01T00:00:00Z",
            &ledger,
        )?;

        let result = persist_transition_application_and_checkpoint(
            &mut conn,
            "transition-1",
            "entity-1",
            "2026-01-01T00:00:01Z",
            "payload-b",
            "checkpoint-b",
            "2026-01-01T00:00:01Z",
            &ledger,
        );
        assert!(matches!(result, Err(JournalDbError::Integrity(_))));

        let conflicting_checkpoint: u64 = conn.query_row(
            "SELECT COUNT(*) FROM ledger_checkpoints WHERE checkpoint_id = 'checkpoint-b'",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(conflicting_checkpoint, 0);
        Ok(())
    }
}
