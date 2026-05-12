//! Process management syscalls
use core::mem::size_of;

use crate::{
    config::PAGE_SIZE,
    mm::{translated_byte_buffer, MapPermission, PageTable, PTEFlags, VirtAddr},
    task::{
        change_program_brk, current_syscall_times, current_user_token, exit_current_and_run_next,
        mmap_current, munmap_current, suspend_current_and_run_next,
    },
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

const TRACE_READ: usize = 0;
const TRACE_WRITE: usize = 1;
const TRACE_SYSCALL: usize = 2;

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    if _ts.is_null() {
        return -1;
    }
    let time_us = get_time_us();
    let time_val = TimeVal {
        sec: time_us / 1_000_000,
        usec: time_us % 1_000_000,
    };
    let src = unsafe {
        core::slice::from_raw_parts((&time_val as *const TimeVal).cast::<u8>(), size_of::<TimeVal>())
    };
    let mut copied = 0;
    for dst in translated_byte_buffer(current_user_token(), _ts as *const u8, size_of::<TimeVal>()) {
        let len = dst.len();
        dst.copy_from_slice(&src[copied..copied + len]);
        copied += len;
    }

    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    match _trace_request {
        TRACE_READ => read_user_byte(_id).map(|byte| byte as isize).unwrap_or(-1),
        TRACE_WRITE => {
            if write_user_byte(_id, _data as u8) {
                0
            } else {
                -1
            }
        }
        TRACE_SYSCALL => current_syscall_times(_id) as isize,
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap");

    if _start % PAGE_SIZE != 0 || _len == 0 {
        return -1;
    }
    if _port == 0 || (_port & !0x7) != 0 {
        return -1;
    }
    if _start.checked_add(_len).is_none() {
        return -1;
    }

    let mut permission = MapPermission::U;
    if (_port & 0x1) != 0 {
        permission |= MapPermission::R;
    }
    if (_port & 0x2) != 0 {
        permission |= MapPermission::W;
    }
    if (_port & 0x4) != 0 {
        permission |= MapPermission::X;
    }

    let map_len = (_len + PAGE_SIZE - 1) / PAGE_SIZE * PAGE_SIZE;

    if mmap_current(_start, map_len, permission) {
        0
    } else {
        -1
    }
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap");

    if _start % PAGE_SIZE != 0 || _len == 0 || _len % PAGE_SIZE != 0 {
        return -1;
    }
    if _start.checked_add(_len).is_none() {
        return -1;
    }

    if munmap_current(_start, _len) {
        0
    } else {
        -1
    }
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

fn read_user_byte(addr: usize) -> Option<u8> {
    let page_table = PageTable::from_token(current_user_token());
    let va = VirtAddr::from(addr);
    let pte = page_table.translate(va.floor())?;
    if !pte.readable() || !pte.flags().contains(PTEFlags::U) {
        return None;
    }
    let ppn = pte.ppn();
    Some(ppn.get_bytes_array()[va.page_offset()])
}

fn write_user_byte(addr: usize, value: u8) -> bool {
    let page_table = PageTable::from_token(current_user_token());
    let va = VirtAddr::from(addr);
    let pte = match page_table.translate(va.floor()) {
        Some(pte) => pte,
        None => return false,
    };
    if !pte.writable() || !pte.flags().contains(PTEFlags::U) {
        return false;
    }
    let ppn = pte.ppn();
    ppn.get_bytes_array()[va.page_offset()] = value;
    true
}
