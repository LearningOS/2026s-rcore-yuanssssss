//! Process management syscalls
use crate::{
    loader::{get_base_i, get_user_stack_range},
    task::current_task_id,
    task::{current_syscall_times, exit_current_and_run_next, suspend_current_and_run_next},
    timer::get_time_us,
    config::APP_SIZE_LIMIT,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

const TRACE_READ: usize = 0;
const TRACE_WRITE: usize = 1;
const TRACE_SYSCALL: usize = 2;

fn current_app_ranges() -> ((usize, usize), (usize, usize)) {
    let task_id = current_task_id();
    let app_base = get_base_i(task_id);
    let app_range = (app_base, app_base + APP_SIZE_LIMIT);
    let stack_range = get_user_stack_range(task_id);
    (app_range, stack_range)
}

fn in_range(addr: usize, range: (usize, usize)) -> bool {
    range.0 <= addr && addr < range.1
}

pub fn sys_trace(trace_request: usize, id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    let (app_range, stack_range) = current_app_ranges();
    match trace_request {
        TRACE_READ => {
            if in_range(id, app_range) || in_range(id, stack_range) {
                unsafe { (id as *const u8).read() as isize }
            } else {
                -1
            }
        }
        TRACE_WRITE => {
            if in_range(id, stack_range) {
                unsafe {
                    (id as *mut u8).write(_data as u8);
                }
                0
            } else {
                -1
            }
        }
        TRACE_SYSCALL => current_syscall_times(id) as isize,
        _ => -1,
    }
}
