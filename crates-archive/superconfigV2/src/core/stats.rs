//! Statistics tracking for the `SuperConfig` V2 registry system

/// Statistics about the registry state
#[derive(Debug, Clone, Default)]
pub struct RegistryStats {
    /// Total number of active handles
    pub total_handles: u64,
    /// Total number of create operations
    pub total_creates: u64,
    /// Total number of read operations
    pub total_reads: u64,
    /// Total number of update operations
    pub total_updates: u64,
    /// Total number of delete operations
    pub total_deletes: u64,
    /// Approximate memory usage in bytes
    pub memory_usage_bytes: u64,
}

impl RegistryStats {
    /// Create new empty statistics
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset all statistics to zero
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Add memory usage
    pub const fn add_memory(&mut self, bytes: u64) {
        self.memory_usage_bytes = self.memory_usage_bytes.saturating_add(bytes);
    }

    /// Remove memory usage
    pub const fn remove_memory(&mut self, bytes: u64) {
        self.memory_usage_bytes = self.memory_usage_bytes.saturating_sub(bytes);
    }

    /// Increment create counter
    pub const fn increment_creates(&mut self) {
        self.total_creates = self.total_creates.saturating_add(1);
        self.total_handles = self.total_handles.saturating_add(1);
    }

    /// Increment read counter
    pub const fn increment_reads(&mut self) {
        self.total_reads = self.total_reads.saturating_add(1);
    }

    /// Increment update counter
    pub const fn increment_updates(&mut self) {
        self.total_updates = self.total_updates.saturating_add(1);
    }

    /// Increment delete counter
    pub const fn increment_deletes(&mut self) {
        self.total_deletes = self.total_deletes.saturating_add(1);
        self.total_handles = self.total_handles.saturating_sub(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_new_and_default() {
        let stats1 = RegistryStats::new();
        let stats2 = RegistryStats::default();

        assert_eq!(stats1.total_handles, 0);
        assert_eq!(stats1.total_creates, 0);
        assert_eq!(stats1.total_reads, 0);
        assert_eq!(stats1.total_updates, 0);
        assert_eq!(stats1.total_deletes, 0);
        assert_eq!(stats1.memory_usage_bytes, 0);

        assert_eq!(stats1.total_handles, stats2.total_handles);
        assert_eq!(stats1.total_creates, stats2.total_creates);
    }

    #[test]
    fn test_stats_operations() {
        let mut stats = RegistryStats::new();

        // Test create operation
        stats.increment_creates();
        assert_eq!(stats.total_creates, 1);
        assert_eq!(stats.total_handles, 1);

        // Test read operation
        stats.increment_reads();
        assert_eq!(stats.total_reads, 1);
        assert_eq!(stats.total_handles, 1); // Should not change

        // Test update operation
        stats.increment_updates();
        assert_eq!(stats.total_updates, 1);
        assert_eq!(stats.total_handles, 1); // Should not change

        // Test delete operation
        stats.increment_deletes();
        assert_eq!(stats.total_deletes, 1);
        assert_eq!(stats.total_handles, 0); // Should decrease
    }

    #[test]
    fn test_memory_tracking() {
        let mut stats = RegistryStats::new();

        stats.add_memory(100);
        assert_eq!(stats.memory_usage_bytes, 100);

        stats.add_memory(50);
        assert_eq!(stats.memory_usage_bytes, 150);

        stats.remove_memory(30);
        assert_eq!(stats.memory_usage_bytes, 120);

        // Test underflow protection
        stats.remove_memory(200);
        assert_eq!(stats.memory_usage_bytes, 0);
    }

    #[test]
    fn test_overflow_protection() {
        let mut stats = RegistryStats::new();
        stats.total_creates = u64::MAX;
        stats.total_handles = u64::MAX;
        stats.memory_usage_bytes = u64::MAX;

        // These should not panic due to saturating operations
        stats.increment_creates();
        stats.add_memory(100);

        assert_eq!(stats.total_creates, u64::MAX);
        assert_eq!(stats.total_handles, u64::MAX);
        assert_eq!(stats.memory_usage_bytes, u64::MAX);
    }

    #[test]
    fn test_reset() {
        let mut stats = RegistryStats::new();

        // Set some values
        stats.increment_creates();
        stats.increment_reads();
        stats.add_memory(1000);

        assert_ne!(stats.total_creates, 0);
        assert_ne!(stats.total_reads, 0);
        assert_ne!(stats.memory_usage_bytes, 0);

        // Reset
        stats.reset();

        assert_eq!(stats.total_creates, 0);
        assert_eq!(stats.total_reads, 0);
        assert_eq!(stats.memory_usage_bytes, 0);
    }
}
