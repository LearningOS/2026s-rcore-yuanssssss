//! Types related to task management

use super::TaskContext;

/// Maximum number of distinct syscalls tracked for one task.
pub const MAX_SYSCALL_NUM: usize = 16;

/// Per-syscall statistics entry.
#[derive(Copy, Clone)]
pub struct SyscallInfo {
    /// Syscall id.
    pub id: usize,
    /// How many times this syscall has been invoked.
    pub times: usize,
}

/// The task control block (TCB) of a task.
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,
    /// Per-task syscall invocation counters stored as a compact linear table.
    pub syscall_info: [SyscallInfo; MAX_SYSCALL_NUM],
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
