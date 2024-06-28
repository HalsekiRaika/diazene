use time::OffsetDateTime;
use crate::persistence::metadata::SnapShotMetadata;

pub enum SnapShotSelectionCriteria {
    Sequence { 
        min: usize, 
        max: usize 
    },
    Timestamp { 
        min: OffsetDateTime, 
        max: OffsetDateTime 
    },
    Both { 
        min_seq: usize,
        max_seq: usize,
        min_time: OffsetDateTime,
        max_time: OffsetDateTime,
    }
}


impl SnapShotSelectionCriteria {
    pub const LATEST: SnapShotSelectionCriteria = SnapShotSelectionCriteria::Sequence { min: usize::MIN, max: usize::MAX };
    
    pub(crate) fn matches(&self, metadata: &SnapShotMetadata) -> bool {
        match self {
            SnapShotSelectionCriteria::Sequence { min, max } 
                => min <= &metadata.sequence && &metadata.sequence <= max,
            SnapShotSelectionCriteria::Timestamp { min, max } 
                => min <= &metadata.timestamp && &metadata.timestamp <= max,
            SnapShotSelectionCriteria::Both { min_seq, max_seq, min_time, max_time } 
                => min_seq <= &metadata.sequence && &metadata.sequence <= max_seq &&
                   min_time <= &metadata.timestamp && &metadata.timestamp <= max_time
        }
    }
}


#[cfg(test)]
mod test {
    use std::collections::BTreeSet;
    use time::{Duration, OffsetDateTime};
    use crate::persistence::criteria::SnapShotSelectionCriteria;
    use crate::persistence::metadata::SnapShotMetadata;
    use crate::persistence::PersistenceId;


    pub fn create_metadata() -> BTreeSet<SnapShotMetadata> {
        let id = PersistenceId::new(uuid::Uuid::new_v4());
        let now = OffsetDateTime::now_utc();
        BTreeSet::from([
            SnapShotMetadata { id: id.clone(), sequence: 0, timestamp: now + Duration::hours(1), metadata: None, },
            SnapShotMetadata { id: id.clone(), sequence: 1, timestamp: now + Duration::hours(2), metadata: None, },
            SnapShotMetadata { id: id.clone(), sequence: 2, timestamp: now + Duration::hours(3), metadata: None, },
        ])
    }
    
    #[test]
    pub fn cmp_seq() {
        let metadata = create_metadata();
        let criteria = SnapShotSelectionCriteria::LATEST;
        
        let latest = metadata.iter().find(|x| criteria.matches(x)).unwrap();
        assert_ne!(latest, metadata.iter().max().unwrap());
        
        let criteria = SnapShotSelectionCriteria::Sequence { min: 1, max: 2 };
        
        let select = metadata.iter().rev().find(|x| criteria.matches(x)).unwrap();
        assert_eq!(select, metadata.iter().find(|x| x.sequence == 2).unwrap());
    }
    
    #[test]
    pub fn cmp_timestamp() {
        let now = OffsetDateTime::now_utc();
        let metadata = create_metadata();
        let max = now + Duration::hours(2) + Duration::minutes(30);
        
        let criteria = SnapShotSelectionCriteria::Timestamp { min: now, max };
        
        let select = metadata.iter().rev().find(|x| criteria.matches(x)).unwrap();
        assert_eq!(select, metadata.iter().find(|x| x.sequence == 1).unwrap());
    }
}