//! Types related to task management

use super::TaskContext;

/// The number of distinct system calls tracked for each task.
const MAX_TRACKED_SYSCALLS: usize = 16;

/// Usage statistics for one system call.
#[derive(Copy, Clone)]
struct SyscallStat {
    id: usize,
    count: usize,
}

/// System call usage statistics for one task.
#[derive(Clone)]
pub struct SyscallStats {
    entries: [SyscallStat; MAX_TRACKED_SYSCALLS],
    len: usize,
}

impl SyscallStats {
    /// Create an empty set of system call statistics.
    pub const fn new() -> Self {
        Self {
            entries: [SyscallStat { id: 0, count: 0 }; MAX_TRACKED_SYSCALLS],
            len: 0,
        }
    }

    /// Record one invocation of the specified system call.
    pub fn record(&mut self, syscall_id: usize) {
        if let Some(entry) = self.entries[..self.len]
            .iter_mut()
            .find(|entry| entry.id == syscall_id)
        {
            entry.count += 1;
            return;
        }

        assert!(
            self.len < self.entries.len(),
            "too many distinct system calls to track"
        );
        self.entries[self.len] = SyscallStat {
            id: syscall_id,
            count: 1,
        };
        self.len += 1;
    }

    /// Return how many times the specified system call was invoked.
    pub fn count(&self, syscall_id: usize) -> usize {
        self.entries[..self.len]
            .iter()
            .find(|entry| entry.id == syscall_id)
            .map_or(0, |entry| entry.count)
    }
}

/// The task control block (TCB) of a task.
#[derive(Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,

    /// System call usage statistics for this task.
    pub syscall_stats: SyscallStats,
}

/// The status of a task
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    /// uninitialized
    UnInit,
    /// ready to run
    Ready,
    /// running
    Running,
    /// exited
    Exited,
}
