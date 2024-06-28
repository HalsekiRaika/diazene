use std::cmp::Ordering;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use crate::persistence::PersistenceId;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SnapShotMetadata {
    pub id: PersistenceId,
    pub sequence: usize,
    pub timestamp: OffsetDateTime,
    pub metadata: Option<flexbuffers::FlexBufferType>
}

impl Eq for SnapShotMetadata {}

impl PartialEq<Self> for SnapShotMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id) 
            && self.sequence.eq(&other.sequence)
            && self.timestamp.eq(&other.timestamp)
    }
}

impl Ord for SnapShotMetadata {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.id.ne(&other.id) {
            return self.id.cmp(&other.id)
        } else if self.sequence.ne(&other.sequence) {
            return self.sequence.cmp(&other.sequence)
        } else if self.timestamp.ne(&other.timestamp) {
            return self.timestamp.cmp(&other.timestamp)
        }
        
        Ordering::Equal
    }
}

impl PartialOrd<Self> for SnapShotMetadata {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
