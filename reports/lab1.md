## Chapter 3 实验报告

### 1. 运行 bad 测例后的现象分析

为了验证用户程序在进入 U 态后是否真的受到了特权级限制，我运行了 `ch2b_bad_*` 相关测例。实验现象表明，用户态程序一旦尝试执行不允许的操作，就会被内核捕获并终止。

以下是处理这些异常的输出

```text
[kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.
[kernel] IllegalInstruction in application, kernel killed it.
```

这说明两类非法行为都会被内核处理：

1. 访问非法地址时，会触发 `PageFault` 或相关访存异常。
2. 执行特权级不允许的指令时，会触发 `IllegalInstruction`。

从内核实现上看，这些异常首先由 trap 机制捕获，然后进入内核的 trap 处理流程，再根据异常类型执行对应处理逻辑。当前实验中的结果是：内核识别出异常后，直接终止对应用户程序。

因此可以说明，用户程序虽然已经能够在 U 态运行，但它仍然受到硬件特权级与内核异常处理机制的约束，不能随意访问非法资源，也不能执行 S 态特权指令。

### 2. 内核如何进入 S 态

#### 2.1 启动阶段的整体流程

机器加电后，并不会立刻开始执行内核，而是先进入 RustSBI。RustSBI 运行在更高特权级，用来完成早期硬件初始化，并在准备完成后将控制权交给内核。

本次实验中，启动流程可以概括为：

1. 处理器从较早的启动入口开始执行，初始可观察到入口位于 `0x1000` 附近，没有找具体的跳转指令了。
2. 之后进入 RustSBI 所在的区域，由 RustSBI 在 M 态完成初始化。
3. RustSBI 准备将控制权交给内核入口 `0x80200000`。

#### 2.2 从 RustSBI 角度看进入 S 态的过程

在阅读 RustSBI 的源码后，可以看到 RustSBI 在初始化完成后，会输出提示信息并调用：

```rust
execute::execute_supervisor(0x80200000, hartid, opqaue, HSM.clone());
```

在这条语句之前，RustSBI 还会打印类似如下信息：

```text
[rustsbi] enter supervisor 0x80200000
```

这说明 RustSBI 已经准备把控制权交给 supervisor，也就是我们的内核。虽然真正设置 CSR 并执行返回指令的细节封装在 `execute_supervisor` 内部，但按照 RISC-V 的标准流程，这一步本质上会完成以下工作：

1. 设置 `mepc = 0x80200000`，指定内核入口地址。
2. 设置 `mstatus.MPP = Supervisor`，表示从 M 态返回时要进入 S 态。
3. 准备好传给内核的参数，例如 `hartid` 和设备树相关信息。
4. 执行 `mret`，从 M 态切换到 S 态，并跳转到 `0x80200000`。



其实主要是打算找一个表示机器状态的CSR，根据CSR的变化设置条件断点
查资料发现这个寄存器叫做mstatus，但是我调试的时候不会用，
然后直接找说明书上面对应的源码去看了，刚好91行就是
```rust
println!("[rustsbi] enter supervisor 0x80200000");
```


## 本次lab

直接大力出奇迹的，感觉正常应该是搜一下trace的系统调用，然后再写。
然后本次是写好直接调，只要过样例就好了。


