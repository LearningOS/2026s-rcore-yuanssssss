# Chapter 3 实验记录

## 任务背景

训练营课件中没有直接给出本次任务说明，因此主要参考 rCore 教材中的实验要求：

<https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter3/5exercise.html#id5>

书中的描述大意是实现一个新的系统调用，用于查询任务信息：

```rust
/// syscall ID: 410

/// 根据任务 ID 查询任务信息，任务信息包括任务 ID、任务状态、
/// 系统调用统计信息以及任务总运行时长。
/// 成功返回 0，失败返回 -1。
fn sys_task_info(id: usize, ts: *mut TaskInfo) -> isize;

struct TaskInfo {
    id: usize,
    status: TaskStatus,
    call: [SyscallInfo; MAX_SYSCALL_NUM],
    time: usize,
}

struct SyscallInfo {
    id: usize,
    times: usize,
}
```

除了编程题外，本章还有两道问答题：

1. 正确进入 U 态后，若执行 S 态特权指令或访问 S 态寄存器，程序会出现什么错误行为。可以通过运行 `ch2b_bad_*.rs` 两个测试来观察，并记录所使用的 SBI 及其版本。
2. 通过 gdb 跟踪或阅读源代码，分析机器从加电到跳转到 `0x80200000` 的过程，并回答内核是如何进入 S 态的。

## 当前仓库中的实际任务

在阅读当前仓库代码时，我搜索 `TODO` 后发现内核中存在一个尚未实现的 `sys_trace`。它与书中提到的 `sys_task_info` 在接口形式上并不完全一致，因此一开始我怀疑这两者不是同一个任务。

继续检查测评代码后，可以确认本仓库当前真正需要完成的，是 `sys_trace` 对应的系统调用逻辑，而不是直接照搬教材中的 `sys_task_info`。

原因如下：

1. `ci-user/user` 中已经存在用户态 `sys_trace` 封装。
2. `sys_trace` 又被进一步封装成了更易用的 `trace` 接口。
3. `ch3_trace.rs` 明确依赖这一套接口进行测试。

因此，对于当前这份仓库来说，本次实验的核心目标可以理解为：

> 实现 `TRACE` 系统调用，为用户态提供查询或观测能力，并通过 `ch3_trace` 的测评。

## 用户态接口分析

在查看 `ci-user/user/src/lib.rs` 后，可以看到用户态已经封装了以下几个接口：

- `count_syscall`
- `trace_read`
- `trace_write`

它们最终都会调用底层的 `trace`，再进入 `sys_trace`。

这说明 `sys_trace` 并不是单一功能接口，而是一个带有多种请求类型的统一入口。

## 初步实现思路

我的第一版理解是：

1. 用户态通过 `ecall` 触发系统调用。
2. 进入内核后，会经过统一的 `syscall` 分发函数。
3. 如果要统计系统调用次数，那么最自然的位置就是在 `syscall` 分发时，对当前系统调用编号进行记录。

也就是说，可以在内核执行具体 syscall 之前，先把该 syscall 的调用计数加一。这样后续 `count_syscall` 查询时，就能返回对应的统计值。

这个思路本身是合理的，因为它抓住了“所有系统调用都会经过统一入口”这一关键事实。

## 当前认识的修正

随着进一步阅读测试代码，我意识到“只实现系统调用次数统计”这个判断并不完整。

虽然 `count_syscall` 的确是 `sys_trace` 的一部分功能，但 `ch3_trace.rs` 里还使用了：

- `trace_read`
- `trace_write`

因此从完整性上讲，最终要实现的并不只是“统计 syscall 次数”，还包括基于 `TraceRequest` 的不同分支处理。

不过，为了降低调试复杂度，可以先把“统计系统调用次数”这一部分单独做好，再继续补齐 `trace_read` 和 `trace_write`。

## 当前进展

目前已经先从“计数功能”入手，尝试在 syscall 分发路径上记录调用次数。

在这个过程中，我发现还有两个关键点需要特别注意：

1. 统计信息不能简单做成一个全局总表，否则前面应用留下的 syscall 记录会污染当前测试结果。
2. `count_syscall(SYSCALL_TRACE)` 这种测试会把本次 `sys_trace` 调用本身也算进去，因此计数时机必须放在足够靠前的位置。

## 当前遇到的问题

虽然已经按上述思路做了修改，但运行测试后出现了新的问题：

- 与时间相关的测试全部失败。
- `trace` 自身的测试也没有通过。

这说明目前的实现还不完整，或者现有逻辑对任务切换、时间统计、请求分支处理等方面产生了影响。

## 下一步计划

解决这些问题

```
[FAIL] not found <get_time OK498103298535626! (\d+)>
[FAIL] not found <Test sleep OK498103298535626!>
[PASS] found <current time_msec = (\d+)>
[FAIL] not found <time_msec = (\d+) after sleeping (\d+) ticks, delta = (\d+)ms!>
[FAIL] not found <Test sleep1 passed498103298535626!>
[FAIL] not found <string from task trace test>
[FAIL] not found <Test trace OK498103298535626!>
```

可能是我的这个trace，实际上还没有实现的原因。
首先是current_time > 0这个断言失败，
调用来自于time::read()函数，可能是库有问题，但是我没做这个的task的时候，测试是通过的，
如何解决这个问题呢？
1. 会不会是读取时间太早了？,然后把系统调用的数组开小一点，发现就过了，
那接下来就可以把数组变成线性表

解决完这个后，出现了新问题
Panicked at src/bin/ch3_trace.rs:22, assertion failed: 3 <= count_syscall(SYSCALL_GETTIMEOFDAY)

```Rust
let t1 = get_time() as usize;
get_time();
sleep(500);
let t2 = get_time() as usize;
let t3 = get_time() as usize;
assert!(3 <= count_syscall(SYSCALL_GETTIMEOFDAY));
```
按照道理来说是有4次，很奇怪，这个是因为还没改成线性表就尝试了，
调用号是410，数组大小只有64，

接下来需要实现trace_read和trace_write，首先看样例，
就是在内存读和写

这里还有一个实现时必须想清楚的问题：判断地址是否合法时，不能只判断“它是不是一个用户地址”，而必须判断“它是不是当前应用自己的地址”。

原因在于，当前 chapter 3 已经是分时系统。虽然某一时刻 CPU 只会运行一个任务，但内存中可能已经同时装入了多个应用。每个应用都会被加载到自己的地址范围中，因此如果 `trace_read` / `trace_write` 只做宽泛的用户地址判断，那么当前任务就有可能读到甚至改写其他任务的内存，这显然是不安全的。

因此，`sys_trace` 在实现 `Read` 和 `Write` 时，应当基于当前正在运行的任务来判断地址范围：

1. 先确定当前任务是谁。
2. 再根据当前任务的编号，计算它对应的应用代码段范围。
3. 同时确定该任务自己的用户栈范围。
4. `trace_read` 只允许读取当前任务自己的代码段或用户栈。
5. `trace_write` 只允许写当前任务自己的用户栈，不能修改代码段，也不能修改其他任务的地址空间。

也就是说，这里真正需要区分的不是“这是不是某个应用的地址”，而是“这是不是当前应用的地址”。只有这样，`trace` 的行为才符合分时系统下最基本的隔离要求。


接下来就是做两个问答题，首先是运行ch2b_bad*.rs,
首先得从ci-user/user里面执行make 命令，出来，然后构建程bin文件
```bash
 make build CHAPTER=2 TEST=2 BASE=1
```
之后copy出来运行就好
