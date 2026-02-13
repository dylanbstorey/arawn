//! TTL tracking for session expiration.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Tracks last access times for TTL-based expiration.
#[derive(Debug)]
pub struct TtlTracker {
    /// Last access time for each session.
    access_times: HashMap<String, Instant>,

    /// TTL duration (None means no expiration).
    ttl: Option<Duration>,
}

impl TtlTracker {
    /// Create a new TTL tracker with the given duration.
    pub fn new(ttl: Option<Duration>) -> Self {
        Self {
            access_times: HashMap::new(),
            ttl,
        }
    }

    /// Record an access for a session (resets its TTL timer).
    pub fn touch(&mut self, session_id: &str) {
        self.access_times.insert(session_id.to_string(), Instant::now());
    }

    /// Check if a session has expired.
    pub fn is_expired(&self, session_id: &str) -> bool {
        match self.ttl {
            None => false,
            Some(ttl) => {
                match self.access_times.get(session_id) {
                    None => true, // No access record = expired
                    Some(last_access) => last_access.elapsed() > ttl,
                }
            }
        }
    }

    /// Remove tracking for a session.
    pub fn remove(&mut self, session_id: &str) {
        self.access_times.remove(session_id);
    }

    /// Get all expired session IDs.
    pub fn get_expired(&self) -> Vec<String> {
        match self.ttl {
            None => Vec::new(),
            Some(ttl) => {
                let now = Instant::now();
                self.access_times
                    .iter()
                    .filter(|(_, last_access)| now.duration_since(**last_access) > ttl)
                    .map(|(id, _)| id.clone())
                    .collect()
            }
        }
    }

    /// Remove all expired entries and return their IDs.
    pub fn drain_expired(&mut self) -> Vec<String> {
        let expired = self.get_expired();
        for id in &expired {
            self.access_times.remove(id);
        }
        expired
    }

    /// Get the number of tracked sessions.
    pub fn len(&self) -> usize {
        self.access_times.len()
    }

    /// Check if there are no tracked sessions.
    pub fn is_empty(&self) -> bool {
        self.access_times.is_empty()
    }

    /// Clear all tracking data.
    pub fn clear(&mut self) {
        self.access_times.clear();
    }

    /// Get the configured TTL.
    pub fn ttl(&self) -> Option<Duration> {
        self.ttl
    }

    /// Update the TTL configuration.
    pub fn set_ttl(&mut self, ttl: Option<Duration>) {
        self.ttl = ttl;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_no_ttl_never_expires() {
        let mut tracker = TtlTracker::new(None);
        tracker.touch("session-1");

        assert!(!tracker.is_expired("session-1"));
        assert!(tracker.get_expired().is_empty());
    }

    #[test]
    fn test_touch_resets_timer() {
        let mut tracker = TtlTracker::new(Some(Duration::from_millis(50)));
        tracker.touch("session-1");

        // Wait a bit
        thread::sleep(Duration::from_millis(30));

        // Touch again to reset
        tracker.touch("session-1");

        // Wait a bit more (total would be >50ms without reset)
        thread::sleep(Duration::from_millis(30));

        // Should not be expired because we touched it
        assert!(!tracker.is_expired("session-1"));
    }

    #[test]
    fn test_expiration() {
        let mut tracker = TtlTracker::new(Some(Duration::from_millis(10)));
        tracker.touch("session-1");

        // Wait for expiration
        thread::sleep(Duration::from_millis(20));

        assert!(tracker.is_expired("session-1"));
        assert_eq!(tracker.get_expired(), vec!["session-1".to_string()]);
    }

    #[test]
    fn test_drain_expired() {
        let mut tracker = TtlTracker::new(Some(Duration::from_millis(10)));
        tracker.touch("session-1");
        tracker.touch("session-2");

        // Wait for expiration
        thread::sleep(Duration::from_millis(20));

        let expired = tracker.drain_expired();
        assert_eq!(expired.len(), 2);
        assert!(tracker.is_empty());
    }

    #[test]
    fn test_remove() {
        let mut tracker = TtlTracker::new(Some(Duration::from_secs(60)));
        tracker.touch("session-1");
        tracker.touch("session-2");

        tracker.remove("session-1");

        assert_eq!(tracker.len(), 1);
        // Removed sessions are considered expired (no access record)
        assert!(tracker.is_expired("session-1"));
    }
}
