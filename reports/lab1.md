# Lab 1 实验报告

## 简答作业

### 1. U 态程序异常行为

正确进入 U 态后，程序还应具有以下特征：使用 S 态特权指令或访问 S 态寄存器时会报错。

请运行三个 `ch2b_bad_*.rs` 测例，描述程序的出错行为，并注明使用的 SBI 及其版本。

RustSBI-QEMU Version 0.2.0-alpha.2

[kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.
[kernel] IllegalInstruction in application, kernel killed it.
[kernel] IllegalInstruction in application, kernel killed it.
第一个访问0地址，触发异常，被trap捕获，结束，2和3是访问特权级寄存器以及指令，触发异常，被trap捕获，结束

### 2. Trap 上下文切换

深入理解 `trap.S` 中 `__alltraps` 和 `__restore` 两个函数的作用，并回答以下问题。

#### 2.1 `__restore` 入口

`L40`：刚进入 `__restore` 时，`sp` 代表什么值？请指出 `__restore` 的两种使用场景。

`Answer`
__restore的参数为cx_addr,是在内核栈中放入的当前应用的trap上下文，然后调用这个函数时，一定会在内核空间，代表内核栈的栈顶地址，也是当前应用trap上下文，
场景1： 任务初始化时，模拟一个应用返回，进入用户态，
场景2： 进入trap之后，返回原来应用执行

#### 2.2 恢复特权级相关寄存器

`L43-L48`：以下汇编代码特殊处理了哪些寄存器？这些寄存器的值对于进入用户态有何意义？请分别解释。

```asm
ld t0, 32*8(sp)
ld t1, 33*8(sp)
ld t2, 2*8(sp)
csrw sstatus, t0
csrw sepc, t1
csrw sscratch, t2
```
`Answer`
TrapContext时按照C的结构体布局的，所有在内存中，依次是x0-x31，然后时sstatus 和sepc
所以就是把sstatus读入t0,sepc读入t1, x2保存的时用户的栈栈指针，
主要就是恢复环境，将进入trap的信息重新写回，，如果是进行系统调用的话，sepc的值加了4，
可以让用户任务从trap的吓一条指令运行，然后将用户栈指针写入sscratch

#### 2.3 恢复通用寄存器

`L50-L56`：以下代码为何跳过 `x2` 和 `x4`？

```asm
ld x1, 1*8(sp)
ld x3, 3*8(sp)
.set n, 5
.rept 27
    LOAD_GP %n
    .set n, n+1
.endr
```
x2是用户的栈指针，在_restore的参数里面已经写入(push_context)，
x4是线程指针，当前还没有使用

#### 2.4 返回用户态

`L60`：执行以下指令后，`sp` 和 `sscratch` 中的值分别有什么意义？

```asm
csrrw sp, sscratch, sp
```

将sp变成用户栈指针，sscratch存储内核栈指针地址

`__restore` 中的状态切换发生在哪一条指令？为什么执行该指令后会进入用户态？
sret，它会根据sstatus中的特定位数，返回到不同的特权级，因为调用时设置的是用户态特权级，
所以会返回用户态

#### 2.5 进入内核态

`L13`：执行以下指令后，`sp` 和 `sscratch` 中的值分别有什么意义？

```asm
csrrw sp, sscratch, sp
```
sp原来是用户栈，交换后变为内核栈，__alltraps的开始时，寄存器信息保存在内核栈里面，

从 U 态进入 S 态发生在哪一条指令？

一开始默认在内核态，go___restore进入用户态，当有外部事件如中断或者异常发生，才进入内核态
所以是由硬件进行响应，而不是主动触发，或者在执行系统调用时的ecall指令触发

## 实现的功能

主要是实现了一个统计系统调用的功能，首先是构建系统调用`SyscallStat`结构体，包含系统调用的id和次数，
然后是一个`SyscallStats`结构体，是`SyscallStat`的集合，内部是一个数组，通过维护这个数组，提供
系统调用的次数的的增加和查询接口，实现`sys_trace`系统调用。这个实验涉及到的操作系统知识非常少，主要是
看代码实现能力。
