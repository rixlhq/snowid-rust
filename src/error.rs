use std::error::Error;
use std::fmt;

/// Represents errors that can occur during `SnowID` operations
#[derive(Debug, Clone, PartialEq)]
pub enum SnowIDError {
    /// Error when node ID exceeds the maximum allowed value
    InvalidNodeId { node_id: u16, max: u16 },
    /// Error when epoch is outside the timestamp range representable by `SnowID`
    InvalidEpoch { epoch: u64, now: u64, max_age_ms: u64 },
}

impl fmt::Display for SnowIDError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::InvalidNodeId { node_id, max } => {
                write!(f, "Node ID {node_id} is invalid. Maximum allowed value is {max}")
            },
            Self::InvalidEpoch { epoch, now, max_age_ms } => {
                write!(
                    f,
                    "Epoch {epoch} is invalid for current time {now}. Epoch must not be in the future or more than {max_age_ms}ms old"
                )
            },
        }
    }
}

impl Error for SnowIDError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let invalid_node = SnowIDError::InvalidNodeId { node_id: 1024, max: 1023 };
        assert_eq!(invalid_node.to_string(), "Node ID 1024 is invalid. Maximum allowed value is 1023");
    }

    #[test]
    fn test_error_debug() {
        let invalid_node = SnowIDError::InvalidNodeId { node_id: 1024, max: 1023 };
        assert!(format!("{invalid_node:?}").contains("InvalidNodeId"));
    }

    #[test]
    fn test_error_clone() {
        let original = SnowIDError::InvalidNodeId { node_id: 1024, max: 1023 };
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_invalid_epoch_display() {
        let invalid_epoch = SnowIDError::InvalidEpoch { epoch: 200, now: 100, max_age_ms: 10 };
        assert!(invalid_epoch.to_string().contains("Epoch 200 is invalid"));
    }
}
