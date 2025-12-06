use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::{NotificationLevel, OutputLine};

/// Get current timestamp in milliseconds since Unix epoch
pub fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Manages the circular buffer of terminal output (max 10,000 lines).
pub struct HistoryBuffer {
    lines: Arc<RwLock<VecDeque<OutputLine>>>,
    max_capacity: usize,
    truncation_warning_shown: Arc<RwLock<bool>>,
}

impl HistoryBuffer {
    /// Default capacity for the history buffer
    pub const DEFAULT_CAPACITY: usize = 10_000;

    /// Create a new history buffer with the specified capacity
    pub fn new(max_capacity: usize) -> Self {
        Self {
            lines: Arc::new(RwLock::new(VecDeque::with_capacity(max_capacity))),
            max_capacity,
            truncation_warning_shown: Arc::new(RwLock::new(false)),
        }
    }

    /// Add line to buffer; evict oldest if at capacity
    pub fn push(&self, line: OutputLine) {
        let mut lines = self.lines.write().unwrap();
        let mut warning_shown = self.truncation_warning_shown.write().unwrap();

        // Compute how many items we will add: 1 for the new line, +1 if warning will be inserted
        let need_warning = lines.len() >= self.max_capacity && !*warning_shown;
        let will_add: usize = 1 + if need_warning { 1 } else { 0 };

        // Pop enough items so that lines.len() + will_add <= max_capacity
        while lines.len() + will_add > self.max_capacity {
            lines.pop_front();
        }

        // Insert truncation warning (once) before the new line
        if need_warning {
            let warning = OutputLine::Notification {
                message: format!(
                    "Output truncated: line limit ({}) exceeded",
                    self.max_capacity
                ),
                level: NotificationLevel::Warning,
                timestamp: current_timestamp_ms(),
            };
            lines.push_back(warning);
            *warning_shown = true;
        }

        lines.push_back(line);
    }

    /// Get all lines for rendering (cloned)
    pub fn get_all(&self) -> Vec<OutputLine> {
        self.lines.read().unwrap().iter().cloned().collect()
    }

    /// Get line count
    pub fn len(&self) -> usize {
        self.lines.read().unwrap().len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.lines.read().unwrap().is_empty()
    }

    /// Clear all lines
    pub fn clear(&self) {
        self.lines.write().unwrap().clear();
        *self.truncation_warning_shown.write().unwrap() = false;
    }

    /// Check if the truncation warning has been shown
    pub fn has_truncation_warning(&self) -> bool {
        *self.truncation_warning_shown.read().unwrap()
    }

    /// Get the first line (if any)
    pub fn first(&self) -> Option<OutputLine> {
        self.lines.read().unwrap().front().cloned()
    }

    /// Check if buffer contains a notification with the given message substring
    pub fn contains_warning(&self, substring: &str) -> bool {
        self.lines.read().unwrap().iter().any(|line| {
            if let OutputLine::Notification { message, .. } = line {
                message.contains(substring)
            } else {
                false
            }
        })
    }
}

impl Default for HistoryBuffer {
    fn default() -> Self {
        Self::new(Self::DEFAULT_CAPACITY)
    }
}

impl Clone for HistoryBuffer {
    fn clone(&self) -> Self {
        Self {
            lines: Arc::clone(&self.lines),
            max_capacity: self.max_capacity,
            truncation_warning_shown: Arc::clone(&self.truncation_warning_shown),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // T007: Unit tests for HistoryBuffer capacity, wraparound, and truncation warning
    #[test]
    fn test_buffer_basic_push_and_get() {
        let buffer = HistoryBuffer::new(100);

        buffer.push(OutputLine::Stdout {
            text: "line1".to_string(),
            timestamp: 1000,
        });
        buffer.push(OutputLine::Stdout {
            text: "line2".to_string(),
            timestamp: 2000,
        });

        assert_eq!(buffer.len(), 2);
        let lines = buffer.get_all();
        assert_eq!(lines[0].text(), "line1");
        assert_eq!(lines[1].text(), "line2");
    }

    #[test]
    fn test_buffer_capacity_enforcement() {
        let buffer = HistoryBuffer::new(3);

        // Add 3 lines
        for i in 0..3 {
            buffer.push(OutputLine::Stdout {
                text: format!("line{}", i),
                timestamp: i as u64 * 1000,
            });
        }

        assert_eq!(buffer.len(), 3);

        // Add 4th line - should evict oldest, add warning, and stay within capacity
        buffer.push(OutputLine::Stdout {
            text: "line3".to_string(),
            timestamp: 3000,
        });

        // Buffer should be at capacity with warning included
        let lines = buffer.get_all();
        assert!(buffer.len() <= 3);
        assert!(buffer.has_truncation_warning());

        // Verify warning is in the buffer
        assert!(lines
            .iter()
            .any(|l| matches!(l, OutputLine::Notification { .. })));
    }

    #[test]
    fn test_buffer_truncation_at_exact_capacity() {
        let buffer = HistoryBuffer::new(5);

        // Fill to exactly capacity
        for i in 0..5 {
            buffer.push(OutputLine::Stdout {
                text: format!("line{}", i),
                timestamp: i as u64 * 1000,
            });
        }

        assert_eq!(buffer.len(), 5);
        assert!(!buffer.has_truncation_warning());

        // One more line triggers truncation
        buffer.push(OutputLine::Stdout {
            text: "line5".to_string(),
            timestamp: 5000,
        });

        assert!(buffer.has_truncation_warning());
        assert!(buffer.contains_warning("truncated"));
    }

    #[test]
    fn test_buffer_truncation_warning_shown_once() {
        // Use a larger buffer so the warning doesn't get evicted immediately
        let buffer = HistoryBuffer::new(20);

        // Fill and trigger truncation multiple times
        for i in 0..30 {
            buffer.push(OutputLine::Stdout {
                text: format!("line{}", i),
                timestamp: i as u64 * 1000,
            });
        }

        // Warning flag should be set
        assert!(buffer.has_truncation_warning());

        // Warning should appear exactly once in buffer
        let lines = buffer.get_all();
        let warning_count = lines
            .iter()
            .filter(|l| matches!(l, OutputLine::Notification { .. }))
            .count();

        assert_eq!(warning_count, 1);
    }

    #[test]
    fn test_buffer_truncation_warning_flag_persists() {
        // Test with tiny buffer where warning gets evicted
        let buffer = HistoryBuffer::new(3);

        // Fill and trigger truncation many times - warning will be evicted
        for i in 0..10 {
            buffer.push(OutputLine::Stdout {
                text: format!("line{}", i),
                timestamp: i as u64 * 1000,
            });
        }

        // Warning flag should still be set even if warning was evicted
        assert!(buffer.has_truncation_warning());
    }

    #[test]
    fn test_buffer_clear() {
        let buffer = HistoryBuffer::new(3);

        buffer.push(OutputLine::Stdout {
            text: "line1".to_string(),
            timestamp: 1000,
        });

        assert_eq!(buffer.len(), 1);

        buffer.clear();

        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        assert!(!buffer.has_truncation_warning());
    }

    #[test]
    fn test_buffer_is_empty() {
        let buffer = HistoryBuffer::new(10);

        assert!(buffer.is_empty());

        buffer.push(OutputLine::Command {
            text: "test".to_string(),
            timestamp: 0,
        });

        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_buffer_first_line() {
        let buffer = HistoryBuffer::new(10);

        assert!(buffer.first().is_none());

        buffer.push(OutputLine::Command {
            text: "first".to_string(),
            timestamp: 1000,
        });
        buffer.push(OutputLine::Stdout {
            text: "second".to_string(),
            timestamp: 2000,
        });

        let first = buffer.first().unwrap();
        assert_eq!(first.text(), "first");
    }

    #[test]
    fn test_buffer_clone_shares_data() {
        let buffer = HistoryBuffer::new(10);
        let cloned = buffer.clone();

        buffer.push(OutputLine::Stdout {
            text: "shared".to_string(),
            timestamp: 1000,
        });

        // Clone should see the same data (Arc sharing)
        assert_eq!(cloned.len(), 1);
        assert_eq!(cloned.first().unwrap().text(), "shared");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // T008: Property-based test for HistoryBuffer wraparound using proptest
    proptest! {
        #[test]
        fn test_buffer_never_exceeds_capacity(
            capacity in 10usize..100,
            num_items in 0usize..500
        ) {
            let buffer = HistoryBuffer::new(capacity);

            for i in 0..num_items {
                buffer.push(OutputLine::Stdout {
                    text: format!("line{}", i),
                    timestamp: i as u64,
                });
            }

            // Buffer should never exceed capacity
            prop_assert!(buffer.len() <= capacity);
        }

        #[test]
        fn test_buffer_maintains_order(
            capacity in 5usize..50,
            num_items in 1usize..100
        ) {
            let buffer = HistoryBuffer::new(capacity);

            for i in 0..num_items {
                buffer.push(OutputLine::Stdout {
                    text: format!("{}", i),
                    timestamp: i as u64,
                });
            }

            let lines = buffer.get_all();
            let non_notification_lines: Vec<_> = lines
                .iter()
                .filter(|l| !matches!(l, OutputLine::Notification { .. }))
                .collect();

            // Timestamps should be in ascending order
            for i in 1..non_notification_lines.len() {
                prop_assert!(
                    non_notification_lines[i].timestamp() >= non_notification_lines[i - 1].timestamp()
                );
            }
        }

        #[test]
        fn test_buffer_truncation_eventually_happens(
            capacity in 5usize..20,
        ) {
            let buffer = HistoryBuffer::new(capacity);

            // Add exactly capacity + 1 items
            for i in 0..=capacity {
                buffer.push(OutputLine::Stdout {
                    text: format!("line{}", i),
                    timestamp: i as u64,
                });
            }

            // Truncation warning should have been shown
            prop_assert!(buffer.has_truncation_warning());
        }
    }
}
