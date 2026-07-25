//! Process management syscalls
use crate::{
    task::{current_task_syscall_count, exit_current_and_run_next, suspend_current_and_run_next},
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
    // 0 read current task memory value, _id represents the address of the memory address, as a *const u8  return the value of the memory address, as a isize
    //1 write current task memory value, _id represents the address of the memory address, as a *mut u8, _data represents the value to be written, return 0
    // 2 read current syscall() count,  _id reprensents the syscall number,   return the count of current task syscall _id   as a isize
    match _trace_request {
        0 => {
            let ptr = _id as *const u8;
            unsafe { *ptr as isize }
        }
        1 => {
            let ptr = _id as *mut u8;
            unsafe { *ptr = _data as u8 };
            0
        }
        2 => current_task_syscall_count(_id),
        _ => -1,
    }
}
