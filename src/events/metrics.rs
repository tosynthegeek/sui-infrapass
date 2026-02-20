use tokio::time::Instant;

#[derive(Debug, Clone)]
pub struct EventMetrics {
    pub last_checkpoint_received: Option<u64>,
    pub last_checkpoint_with_event: Option<u64>,
    pub last_checkpoint_received_at: Option<Instant>,
    pub last_event_seen_at: Option<Instant>,
    pub total_checkpoints_processed: u64,
    pub total_events_processed: u64,
    pub connection_healthy: bool,
}

impl Default for EventMetrics {
    fn default() -> Self {
        Self {
            last_checkpoint_received: None,
            last_checkpoint_with_event: None,
            last_checkpoint_received_at: None,
            last_event_seen_at: None,
            total_checkpoints_processed: 0,
            total_events_processed: 0,
            connection_healthy: false,
        }
    }
}
