# 使用 VS Code 调试 rCore

本项目支持在 VS Code 中构建内核、启动 QEMU GDB Server，并使用源码断点调试 rCore。

## 环境要求

开始调试前，请确认已经安装以下工具：

- VS Code
- VS Code 扩展 `C/C++`（`ms-vscode.cpptools`）
- `gdb-multiarch`
- `qemu-system-riscv64`
- 项目所需的 Rust 工具链

在 Ubuntu 24.04 中可以安装 GDB：

```bash
sudo apt update
sudo apt install gdb-multiarch
```

调试任务会依次在以下位置查找 QEMU：

1. `~/qemu-7.0.0/build/qemu-system-riscv64`
2. 当前环境变量 `PATH` 中的 `qemu-system-riscv64`

可以使用以下命令检查工具：

```bash
gdb-multiarch --version
qemu-system-riscv64 --version
```

如果 QEMU 只存在于源码构建目录中，请执行：

```bash
~/qemu-7.0.0/build/qemu-system-riscv64 --version
```

## 开始调试

1. 使用 VS Code 打开仓库根目录。
2. 根据扩展推荐安装 `C/C++` 和 `rust-analyzer`。
3. 在 Rust 源文件的行号左侧单击以设置断点。
4. 打开侧边栏中的“运行和调试”。
5. 选择 `Debug rCore kernel (QEMU)`。
6. 按 `F5` 启动调试。

启动后，VS Code 会自动完成以下操作：

```text
构建用户程序
  -> 构建带调试信息的内核 ELF
  -> 生成内核二进制 os.bin
  -> 启动暂停状态的 QEMU
  -> 使用 gdb-multiarch 连接 127.0.0.1:1234
  -> 在 rust_main 处停止
```

QEMU 的串口输出显示在 VS Code 的任务终端中。结束调试时，VS Code 会停止本次调试启动的 QEMU 进程。

## 调试操作

连接成功后，可以使用 VS Code 调试工具栏进行以下操作：

- 继续运行：`F5`
- 单步跳过：`F10`
- 单步进入：`F11`
- 单步跳出：`Shift+F11`
- 停止调试：`Shift+F5`

“变量”“监视”“调用堆栈”和“断点”面板可以用于检查当前内核状态。切换到反汇编视图后，还可以进行指令级调试。

## 选择用户测试程序

VS Code 默认执行：

```bash
make -C os build OFFLINE=1
```

当前分支为 `ch3` 时，Makefile 默认使用 `TEST=3 BASE=1`，即构建截至 ch3 的基础测试。要改变测试范围，可以修改 `.vscode/tasks.json` 中 `rCore: Build kernel` 任务的参数，例如同时构建普通测试和基础测试：

```json
"args": [
    "-C",
    "${workspaceFolder}/os",
    "build",
    "OFFLINE=1",
    "TEST=3",
    "BASE=2"
]
```

其中：

- `BASE=0`：普通测试，例如 `ch3_trace.rs`
- `BASE=1`：基础测试，例如 `ch3b_yield0.rs`
- `BASE=2`：普通测试和基础测试

## 配置文件

- `.vscode/launch.json`：配置 GDB、内核 ELF、远程调试地址和入口断点。
- `.vscode/tasks.json`：构建项目以及启动、停止 QEMU。
- `.vscode/extensions.json`：声明推荐安装的 VS Code 扩展。
- `os/Cargo.toml`：通过 `[profile.release] debug = true` 保留 DWARF 源码调试信息。

QEMU 实际加载的是经过 `objcopy --strip-all` 处理的 `os.bin`，GDB 则读取包含符号和源码行号的内核 ELF。调试信息不会改变加载到虚拟机中的内核二进制布局。

## 常见问题

### 找不到 `gdb-multiarch`

确认 `/usr/bin/gdb-multiarch` 存在：

```bash
command -v gdb-multiarch
```

如果安装位置不同，需要修改 `.vscode/launch.json` 中的 `miDebuggerPath`。

### 找不到 `qemu-system-riscv64`

确认 QEMU 已安装到 `PATH`，或者位于：

```text
~/qemu-7.0.0/build/qemu-system-riscv64
```

如果使用其他源码目录，需要修改 `.vscode/tasks.json` 中为 QEMU 任务设置的 `PATH`。

### 无法连接端口 1234

可能已有 QEMU 占用了 GDB 端口。先执行 VS Code 任务：

```text
rCore: Stop QEMU GDB server
```

然后重新按 `F5`。也可以检查端口占用：

```bash
ss -ltnp | grep 1234
```

### 断点显示为灰色或无法命中

确认使用的是 VS Code 调试任务生成的 release ELF，并重新构建：

```bash
make -C os build OFFLINE=1
```

内核 ELF 应包含 DWARF 信息：

```bash
readelf -S os/target/riscv64gc-unknown-none-elf/release/os \
    | grep -E '\.debug_(info|line)'
```

优化后的代码可能被内联或删除。遇到这种情况，可以先在 `rust_main`、未内联函数或汇编指令处设置断点。
