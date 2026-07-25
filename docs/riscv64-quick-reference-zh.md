# RISC-V 64 位汇编简明指南

本文面向 rCore 代码阅读，重点介绍本项目中常见的 RV64 指令、寄存器、调用约定和 Trap 流程。RV64 的通用寄存器宽度是 64 位（8 字节）；但在 RISC-V 术语中，`word` 仍指 32 位，`doubleword` 才指 64 位。

## 1. 先记住三个基本规则

1. RISC-V 有 32 个通用寄存器 `x0` 到 `x31`，每个寄存器在 RV64 中是 64 位。
2. 普通算术指令只操作寄存器；访问内存需要使用 `ld`、`sd` 等加载/存储指令。
3. `sp`、`a0`、`ra` 等都是 `x` 寄存器的 ABI 别名，并不是额外的寄存器。

例如：

```asm
addi sp, sp, -16  # sp = sp - 16
sd   ra, 8(sp)    # 把 ra 写入地址 sp + 8
ld   ra, 8(sp)    # 从地址 sp + 8 读回 ra
addi sp, sp, 16   # sp = sp + 16
```

内存操作采用 `偏移量(基址寄存器)`：

```text
8(sp) 表示内存地址 sp + 8，而不是 sp 中的第 8 个元素。
```

## 2. 通用寄存器与 ABI 名称

| 寄存器 | ABI 名称 | 典型用途 | 跨函数调用后是否保证不变 |
|---|---|---|---|
| `x0` | `zero` | 常量 0，写入无效 | 固定为 0 |
| `x1` | `ra` | 返回地址 | 否 |
| `x2` | `sp` | 栈顶指针 | 是，由调用者维护栈平衡 |
| `x3` | `gp` | 全局指针 | 是 |
| `x4` | `tp` | 线程指针 | 是 |
| `x5-x7` | `t0-t2` | 临时值 | 否 |
| `x8-x9` | `s0-s1` | 保存值；`s0` 也可作帧指针 | 是 |
| `x10-x17` | `a0-a7` | 参数；`a0-a1` 也放返回值 | 否 |
| `x18-x27` | `s2-s11` | 保存值 | 是 |
| `x28-x31` | `t3-t6` | 临时值 | 否 |

“是否保证不变”由 RISC-V ABI 规定：

- `t*`、`a*` 是 caller-saved。调用者若还需要原值，应在 `call` 前自行保存。
- `s*` 是 callee-saved。被调用函数若要修改它们，必须先保存并在返回前恢复。
- 前 8 个整数参数依次放入 `a0-a7`；整数返回值通常放入 `a0`，必要时也使用 `a1`。

因此 Rust 函数 `fn f(x: usize) -> usize` 通常从 `a0` 取得 `x`，并把返回值放回 `a0`。

## 3. 本项目常见指令

### 数据移动与内存访问

```asm
mv   a0, sp       # a0 = sp；伪指令
li   t0, 100      # t0 = 100；伪指令
la   sp, symbol   # sp = symbol 的地址；伪指令
ld   t0, 16(sp)   # 从 sp + 16 读取 8 字节到 t0
sd   t0, 16(sp)   # 将 t0 的 8 字节写入 sp + 16
```

`mv rd, rs` 通常会被汇编器展开成 `addi rd, rs, 0`。`la`、`li` 也可能展开成多条真实指令。

### 算术、位运算与比较

```asm
add  t0, t1, t2   # t0 = t1 + t2
sub  t0, t1, t2   # t0 = t1 - t2
addi sp, sp, -272 # sp = sp - 272
and  t0, t1, t2
or   t0, t1, t2
xor  t0, t1, t2
slli t0, t1, 3    # t0 = t1 << 3
```

带 `i` 的指令通常有一个立即数操作数，例如 `addi`。立即数范围有限，较大的常量通常由伪指令展开完成。

### 跳转和函数调用

```asm
call function     # ra = 返回地址，然后跳到 function；伪指令
ret               # 跳到 ra；伪指令
j    label        # 无条件跳转；伪指令
beq  t0, t1, L1   # t0 == t1 时跳转
bne  t0, t1, L1   # t0 != t1 时跳转
```

`call` 会改写 `ra`。函数若还要调用其他函数，通常需要先把自己的 `ra` 保存到栈中。

### 同步和环境指令

```asm
ecall    # 请求更高特权级处理服务
sret     # 从 S 模式 Trap 返回，恢复到 sstatus.SPP 指定的特权级
fence.i  # 使之前写入的指令对后续取指可见
wfi      # 等待中断
```

## 4. 栈与结构体偏移

RISC-V 的栈通常从高地址向低地址增长，所以分配栈空间使用减法，释放使用加法：

```asm
addi sp, sp, -34*8  # 为 34 个 usize 分配空间
sd   x1, 1*8(sp)    # 保存 x1 到第 1 个槽位
ld   x1, 1*8(sp)    # 恢复 x1
addi sp, sp, 34*8   # 释放空间
```

本项目的 `TrapContext` 使用 `#[repr(C)]`：

```rust
pub struct TrapContext {
    pub x: [usize; 32],
    pub sstatus: Sstatus,
    pub sepc: usize,
}
```

因此其布局可按固定偏移理解：

```text
0*8(sp)  ... 31*8(sp)  -> x[0] ... x[31]
32*8(sp)                -> sstatus
33*8(sp)                -> sepc
```

例如 `ld t2, 2*8(sp)` 读取的是 `x[2]`，也就是 Trap 发生前的用户栈指针。

## 5. 特权级和 CSR

rCore 主要涉及：

```text
U 模式：用户程序
S 模式：操作系统内核
M 模式：机器固件，QEMU 中通常由 RustSBI 提供
```

CSR 是控制和状态寄存器，不能用普通 `ld`、`sd` 访问。常见 CSR：

| CSR | 作用 |
|---|---|
| `sstatus` | S 模式状态，包括中断状态和 Trap 前的特权级 |
| `sepc` | Trap 返回后执行的指令地址 |
| `scause` | Trap 原因 |
| `stval` | Trap 的附加信息，如出错地址 |
| `stvec` | S 模式 Trap 入口地址 |
| `sscratch` | S 模式临时寄存器；rCore 用它辅助交换用户栈和内核栈 |

常见 CSR 指令：

```asm
csrr  t0, sstatus       # t0 = sstatus
csrw  sstatus, t0       # sstatus = t0
csrrw sp, sscratch, sp  # 原子交换：sp 取得旧 sscratch，sscratch 取得旧 sp
```

## 6. 用 `trap.S` 理解 Trap

用户态发生系统调用或异常时，CPU 会进入 S 模式并跳到 `stvec`，但 CPU 不会自动把 32 个通用寄存器都保存到内核栈。`__alltraps` 必须完成这件事。

进入 Trap 时，rCore 约定：

```text
sp       = 用户栈顶
sscratch = 内核栈顶
```

第一条指令交换二者：

```asm
csrrw sp, sscratch, sp
```

交换后：

```text
sp       = 内核栈顶
sscratch = 用户栈顶
```

随后在内核栈分配 `TrapContext`，保存通用寄存器、`sstatus`、`sepc`，并从 `sscratch` 取出原用户 `sp` 保存到 `x[2]`。最后：

```asm
mv   a0, sp
call trap_handler
```

按照 ABI，`a0` 是第一个参数，所以这里调用的是：

```rust
trap_handler(cx: &mut TrapContext)
```

处理完成后，`trap_handler` 的返回值仍通过 `a0` 返回。`__restore` 先执行：

```asm
mv sp, a0
```

此时 `sp` 指向内核栈上的 `TrapContext`，后续便可用 `偏移(sp)` 恢复寄存器。最终将用户 `sp` 写入 `sscratch`，释放 `TrapContext`，再次交换 `sp` 和 `sscratch`，再执行 `sret` 返回用户态。

## 7. 阅读汇编的实用方法

阅读每条指令时，可以机械地翻译成赋值或内存操作：

```text
mv sp, a0            -> sp = a0
ld t2, 16(sp)        -> t2 = *(u64 *)(sp + 16)
sd t2, 16(sp)        -> *(u64 *)(sp + 16) = t2
addi sp, sp, -272    -> sp = sp - 272
csrw sscratch, t2    -> sscratch = t2
```

然后持续记录少数关键寄存器当前代表什么。例如阅读 `__restore` 时只需先跟踪：

```text
a0       -> TrapContext 地址
sp       -> 当前恢复基址，之后变为内核栈顶，最终变为用户栈顶
t0/t1/t2 -> 临时保存 sstatus、sepc、用户 sp
sscratch -> 用户 sp，交换后保存内核 sp
```

不要把寄存器名与固定含义完全绑定。`a0` 只是在 ABI 中用于参数和返回值；进入函数后，只要遵守调用约定，汇编代码可以把它用于其他目的。

## 8. 最小速查表

```text
参数/返回值：a0-a7 / a0-a1
返回地址：   ra
当前栈顶：   sp
临时寄存器： t0-t6（调用后可能改变）
保存寄存器： s0-s11（被调用者负责恢复）

mv rd, rs        rd = rs
ld rd, n(rs)     rd = memory[rs + n]，读取 8 字节
sd rd, n(rs)     memory[rs + n] = rd，写入 8 字节
addi rd, rs, n   rd = rs + n
call f           调用 f，返回地址写入 ra
ret              返回 ra 指向的位置
csrr/csrw        读/写 CSR
csrrw            读取 CSR，同时写入新值
sret              从 S 模式 Trap 返回
```

进一步学习时，优先查阅 RISC-V ISA 的 RV64I 基础整数指令集、RISC-V psABI 调用约定，以及特权级规范中的 S 模式 CSR 和 Trap 章节。
