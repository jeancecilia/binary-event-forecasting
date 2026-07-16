//! Deterministic message framing (IPC-001).
//!
//! IPC uses:
//! - 4-byte big-endian unsigned length header
//! - UTF-8 JSON payload
//! - Explicit schema version
//! - Maximum frame length (MAX_SIGNAL_FRAME_BYTES)
//! - Read timeout and idle timeout

use crate::MAX_SIGNAL_FRAME_BYTES;

/// Read a framed message from a byte stream.
///
/// Returns `None` if the connection should be closed (oversized frame or EOF).
pub fn read_frame(reader: &mut impl std::io::Read) -> std::io::Result<Option<Vec<u8>>> {
    // Read 4-byte header
    let mut header = [0u8; 4];
    match reader.read_exact(&mut header) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }

    let declared_len = u32::from_be_bytes(header) as usize;

    // Safety check: reject oversized frames before allocating
    if declared_len > MAX_SIGNAL_FRAME_BYTES {
        tracing::warn!(
            declared_len,
            max = MAX_SIGNAL_FRAME_BYTES,
            "Rejecting oversized frame"
        );
        return Ok(None);
    }

    // Allocate and read payload
    let mut payload = vec![0u8; declared_len];
    reader.read_exact(&mut payload)?;

    Ok(Some(payload))
}

/// Serialize a message into a framed byte vector.
pub fn write_frame(payload: &[u8]) -> Vec<u8> {
    assert!(
        payload.len() <= MAX_SIGNAL_FRAME_BYTES,
        "Payload size {} exceeds MAX_SIGNAL_FRAME_BYTES",
        payload.len()
    );

    let len = payload.len() as u32;
    let mut frame = Vec::with_capacity(4 + payload.len());
    frame.extend_from_slice(&len.to_be_bytes());
    frame.extend_from_slice(payload);
    frame
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_write_and_read_frame() {
        let payload = b"{\"test\": true}";
        let frame = write_frame(payload);

        let mut cursor = std::io::Cursor::new(&frame);
        let result = read_frame(&mut cursor).unwrap().unwrap();
        assert_eq!(&result, payload);
    }

    #[test]
    fn test_oversized_frame_rejected() {
        // Declare a frame larger than MAX_SIGNAL_FRAME_BYTES
        let oversized_len = (MAX_SIGNAL_FRAME_BYTES + 1) as u32;
        let header = oversized_len.to_be_bytes();

        let mut cursor = std::io::Cursor::new(&header);
        let result = read_frame(&mut cursor).unwrap();
        // Should return None (connection close) without allocating
        assert!(result.is_none());
    }
}
