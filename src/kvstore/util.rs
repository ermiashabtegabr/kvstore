use std::time::Duration;

pub const _BUFFER_SIZE: usize = 10000;
pub const ELECTION_TIMEOUT: Duration = Duration::from_millis(100);
pub const OUTGOING_MESSAGE_PERIOD: Duration = Duration::from_millis(1);

pub const _WAIT_LEADER_TIMEOUT: Duration = Duration::from_millis(500);
pub const _WAIT_DECIDED_TIMEOUT: Duration = Duration::from_millis(50);
