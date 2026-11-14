# Windows-Subsystem-for-BSD
**Windows Subsystem for BSD (WSB)** is a lightweight compatibility and interop environment that allows BSD userland binaries, shells, and tools to run alongside a Windows host server. It combines a custom Rust interop bridge, a runtime **Binaric Assembler** with Dual-ABI translation, and a C/C++ management CLI (`wsb`).
---

## Architecture Overview

WSB operates across three core components:

1. **WSB CLI & Host Server (`wsb`, `cr-server`):**
   - Manages the BSD user-space lifecycle, configuration, and host interop daemon.
   - Provides commands to start/stop the server, launch a BSD shell, check subsystem status, inspect settings, and uninstall the environment.
2. **Rust Server Bridge (`wsb_interop`):**
   - Connects BSD guest processes to the Windows host server binary over a Unix domain socket (`/var/run/wsb_server.sock`) using length-prefixed `bincode` framing.
   - Exposes C-ABI FFI hooks (`cdylib` / `staticlib`) so native BSD C/C++ programs can trigger host actions such as executing Windows binaries or mounting host drives.
3. **Binaric Assembler & JIT Engine (`wsb_asm`):**
   - Emits and patches x86_64 machine instructions at runtime with strict **W^X (Write XOR Execute)** memory protection via `mmap` and `mprotect`.
   - Generates **System V AMD64 to Microsoft x64 ABI trampolines** (remapping argument registers and allocating 32-byte Windows shadow space) to bridge BSD and Windows calling conventions seamlessly.

---

## Project Structure

```text
wsb/
├── cli/                    # C/C++ WSB CLI entrypoint (main.c) and shell integration (shell.cpp)
├── crates/
│   ├── wsb_interop/        # Rust IPC client library & C-ABI FFI bridge
│   └── wsb_binaric_asm/    # Runtime x86_64 Binaric Assembler & Dual-ABI JIT engine
├── targets/
│   └── x86_64-wsb-freebsd.json  # Custom Rust target specification for WSB
└── README.md
```

---

## Prerequisites

- **Host OS:** Windows 10/11 (x64)
- **Toolchain:**
  - Rust (`nightly` toolchain with `rust-src` component for custom target builds)
  - Clang / GCC / MSVC with C++17 support
  - `lld` linker (`rust-lld`)

---

## Building from Source

### 1. Build the Rust Interop & Binaric Assembler Libraries

Install the nightly toolchain and `rust-src` if compiling against the custom WSB target specification:

```bash
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly
```

Build the interop library (`libwsb.so` / `libwsb.a`):

```bash
cd crates/wsb_interop
cargo +nightly build -Z build-std=core,std,alloc,panic_abort \
  --target ../../targets/x86_64-wsb-freebsd.json \
  --release
```

Build the Binaric Assembler (`libwsb_asm.so` / `libwsb_asm.a`):

```bash
cd ../wsb_binaric_asm
cargo build --release
```

### 2. Build the WSB CLI & Shell

Compile the C/C++ CLI and link against the generated Rust libraries:

```bash
cd ../../cli
cc -O2 main.c -o wsb -L../crates/wsb_interop/target/release -lwsb
```

---

## CLI Usage

The `wsb` command manages the BSD user-space environment and background server:

```text
Usage: wsb <command> [options]

Commands:
  server-start    Start the WSB host interop server (cr-server)
  shell           Launch an interactive BSD user-space shell
  status          Check the running state of the WSB server and environment
  settings        View or update WSB subsystem configuration
  collect         Collect diagnostic logs and environment state
  version         Display the current WSB version
  uninstall       Remove the WSB subsystem and clean up runtime files
```

### Example Workflow

```bash
# Start the background interop server
wsb server-start

# Verify subsystem status
wsb status

# Enter the BSD shell environment
wsb shell
```

---

## FFI & Interop Integration

### C-ABI Server Bridge (`wsb_interop`)
Native BSD binaries can link against `libwsb` to communicate with the Windows host server:

```c
#include <stdint.h>

extern int32_t wsb_ping_server(void);

int main(void) {
    if (wsb_ping_server() == 0) {
        // WSB host server is reachable
    }
    return 0;
}
```

### Binaric Assembler (`wsb_asm`)
Dynamic Dual-ABI trampolines can be assembled and sealed into executable pages from C++:

```cpp
#include <cstdint>

extern "C" {
    void* wsb_asm_create(void);
    void wsb_asm_emit_trampoline(void* asm_ptr, uint64_t win64_fn_addr);
    void* wsb_asm_finalize(void* asm_ptr);
    const uint8_t* wsb_region_entry(const void* region);
    void wsb_region_free(void* region);
}
```

---
