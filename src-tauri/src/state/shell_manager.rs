use std::sync::Arc;
use tokio::process::Child;
use tokio::sync::Mutex;

use super::HistoryBuffer;

/// Tracks the current state of the shell process.
pub struct ShellState {
    /// Current running process (if any)
    pub process: Arc<Mutex<Option<Child>>>,
    /// Process ID of running command
    pub pid: Arc<Mutex<Option<u32>>>,
    /// Current working directory
    pub cwd: Arc<Mutex<String>>,
    /// Is shell currently executing a command?
    pub is_busy: Arc<Mutex<bool>>,
}

impl ShellState {
    /// Create a new shell state with the given initial working directory
    pub fn new(initial_cwd: String) -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
            pid: Arc::new(Mutex::new(None)),
            cwd: Arc::new(Mutex::new(initial_cwd)),
            is_busy: Arc::new(Mutex::new(false)),
        }
    }

    /// Get the current working directory
    pub async fn get_cwd(&self) -> String {
        self.cwd.lock().await.clone()
    }

    /// Set the current working directory
    pub async fn set_cwd(&self, cwd: String) {
        *self.cwd.lock().await = cwd;
    }

    /// Check if shell is currently busy
    pub async fn is_busy(&self) -> bool {
        *self.is_busy.lock().await
    }

    /// Set the busy state
    pub async fn set_busy(&self, busy: bool) {
        *self.is_busy.lock().await = busy;
    }

    /// Atomically try to set busy state from false to true.
    /// Returns true if successfully transitioned from false to true,
    /// false if already busy.
    pub async fn try_set_busy(&self) -> bool {
        let mut is_busy = self.is_busy.lock().await;
        if *is_busy {
            false
        } else {
            *is_busy = true;
            true
        }
    }

    /// Get the current process ID (if any)
    pub async fn get_pid(&self) -> Option<u32> {
        *self.pid.lock().await
    }

    /// Atomically get the PID only if shell is busy.
    /// Acquires both locks to avoid TOCTOU race between is_busy and get_pid.
    #[allow(clippy::doc_markdown)]
    pub async fn get_pid_if_busy(&self) -> Option<u32> {
        // Lock both in consistent order to avoid deadlock
        let is_busy = self.is_busy.lock().await;
        let pid = self.pid.lock().await;
        if *is_busy {
            *pid
        } else {
            None
        }
    }

    /// Set the current process and its PID
    pub async fn set_process(&self, child: Child) {
        let pid = child.id();
        *self.process.lock().await = Some(child);
        *self.pid.lock().await = pid;
    }

    /// Take the current process (removing it from state)
    pub async fn take_process(&self) -> Option<Child> {
        let process = self.process.lock().await.take();
        *self.pid.lock().await = None;
        process
    }

    /// Clear the current process reference
    pub async fn clear_process(&self) {
        *self.process.lock().await = None;
        *self.pid.lock().await = None;
    }
}

impl Default for ShellState {
    fn default() -> Self {
        let initial_cwd = std::env::current_dir()
            .map_or_else(|_| "/".to_string(), |p| p.to_string_lossy().to_string());
        Self::new(initial_cwd)
    }
}

impl Clone for ShellState {
    fn clone(&self) -> Self {
        Self {
            process: Arc::clone(&self.process),
            pid: Arc::clone(&self.pid),
            cwd: Arc::clone(&self.cwd),
            is_busy: Arc::clone(&self.is_busy),
        }
    }
}

/// Manages shell process lifecycle and history buffer
pub struct ShellManager {
    /// Shell state (process, cwd, busy flag)
    pub shell_state: ShellState,
    /// History buffer for terminal output
    pub history_buffer: HistoryBuffer,
}

impl ShellManager {
    /// Create a new shell manager
    pub fn new() -> Self {
        Self {
            shell_state: ShellState::default(),
            history_buffer: HistoryBuffer::default(),
        }
    }

    /// Create a new shell manager with an explicit initial working directory
    pub fn new_with_cwd(initial_cwd: String) -> Self {
        Self {
            shell_state: ShellState::new(initial_cwd),
            history_buffer: HistoryBuffer::default(),
        }
    }

    /// Create a new shell manager with custom capacity
    pub fn with_capacity(buffer_capacity: usize) -> Self {
        Self {
            shell_state: ShellState::default(),
            history_buffer: HistoryBuffer::new(buffer_capacity),
        }
    }

    /// Create a new shell manager with custom capacity and explicit CWD
    pub fn with_capacity_and_cwd(buffer_capacity: usize, initial_cwd: String) -> Self {
        Self {
            shell_state: ShellState::new(initial_cwd),
            history_buffer: HistoryBuffer::new(buffer_capacity),
        }
    }

    /// Get the current working directory
    pub async fn get_cwd(&self) -> String {
        self.shell_state.get_cwd().await
    }

    /// Check if shell is currently busy
    pub async fn is_busy(&self) -> bool {
        self.shell_state.is_busy().await
    }

    /// Atomically try to set busy state from false to true.
    /// Returns true if successfully transitioned from false to true,
    /// false if already busy.
    pub async fn try_set_busy(&self) -> bool {
        self.shell_state.try_set_busy().await
    }

    /// Get the current process ID (if a command is running).
    /// This is atomic - avoids TOCTOU race between busy check and PID retrieval.
    pub async fn get_running_pid(&self) -> Option<u32> {
        self.shell_state.get_pid_if_busy().await
    }
}

impl Default for ShellManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ShellManager {
    fn clone(&self) -> Self {
        Self {
            shell_state: self.shell_state.clone(),
            history_buffer: self.history_buffer.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_shell_state_default_cwd() {
        let state = ShellState::default();
        let cwd = state.get_cwd().await;
        assert!(!cwd.is_empty());
    }

    #[tokio::test]
    async fn test_shell_state_set_cwd() {
        let state = ShellState::new("/tmp".to_string());
        assert_eq!(state.get_cwd().await, "/tmp");

        state.set_cwd("/home/user".to_string()).await;
        assert_eq!(state.get_cwd().await, "/home/user");
    }

    #[tokio::test]
    async fn test_shell_state_busy_flag() {
        let state = ShellState::default();
        assert!(!state.is_busy().await);

        state.set_busy(true).await;
        assert!(state.is_busy().await);

        state.set_busy(false).await;
        assert!(!state.is_busy().await);
    }

    #[tokio::test]
    async fn test_shell_manager_creation() {
        let manager = ShellManager::new();
        assert!(!manager.is_busy().await);
        assert!(manager.history_buffer.is_empty());
    }

    #[tokio::test]
    async fn test_shell_manager_with_custom_capacity() {
        let manager = ShellManager::with_capacity(100);
        assert_eq!(manager.history_buffer.len(), 0);
    }

    #[tokio::test]
    async fn test_shell_manager_clone_shares_state() {
        let manager = ShellManager::new();
        let cloned = manager.clone();

        manager.shell_state.set_busy(true).await;
        assert!(cloned.is_busy().await);
    }

    #[tokio::test]
    async fn test_get_pid_if_busy_returns_none_when_not_busy() {
        let state = ShellState::default();
        *state.pid.lock().await = Some(1234);
        // Not busy, so should return None even though PID is set
        assert!(state.get_pid_if_busy().await.is_none());
    }

    #[tokio::test]
    async fn test_get_pid_if_busy_returns_pid_when_busy() {
        let state = ShellState::default();
        *state.pid.lock().await = Some(5678);
        state.set_busy(true).await;
        // Busy, so should return the PID
        assert_eq!(state.get_pid_if_busy().await, Some(5678));
    }

    #[tokio::test]
    async fn test_get_running_pid_atomic() {
        let manager = ShellManager::new();
        // Set PID but not busy
        *manager.shell_state.pid.lock().await = Some(9999);
        assert!(manager.get_running_pid().await.is_none());

        // Now set busy
        manager.shell_state.set_busy(true).await;
        assert_eq!(manager.get_running_pid().await, Some(9999));
    }

    #[tokio::test]
    async fn test_try_set_busy_returns_true_when_not_busy() {
        let state = ShellState::default();
        assert!(!state.is_busy().await);

        let result = state.try_set_busy().await;
        assert!(result);
        assert!(state.is_busy().await);
    }

    #[tokio::test]
    async fn test_try_set_busy_returns_false_when_already_busy() {
        let state = ShellState::default();
        state.set_busy(true).await;
        assert!(state.is_busy().await);

        let result = state.try_set_busy().await;
        assert!(!result);
        assert!(state.is_busy().await);
    }

    #[tokio::test]
    async fn test_try_set_busy_manager_delegates_correctly() {
        let manager = ShellManager::new();
        assert!(!manager.is_busy().await);

        let result = manager.try_set_busy().await;
        assert!(result);
        assert!(manager.is_busy().await);

        // Try again when already busy
        let result2 = manager.try_set_busy().await;
        assert!(!result2);
        assert!(manager.is_busy().await);

        // Clean up
        manager.shell_state.set_busy(false).await;
        assert!(!manager.is_busy().await);
    }
}
