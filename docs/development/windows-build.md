# Building Communitas on Windows

This guide covers building Communitas from source on Windows systems.

## Prerequisites

### Required Software

1. **Visual Studio 2022 Build Tools** (or full Visual Studio 2022)
   - Required for C/C++ compilation
   - Must include "Desktop development with C++" workload

2. **CMake 3.20+**
   - Required by `aws-lc-sys` for building the AWS Libcrypto library
   - Can be installed via Visual Studio, standalone installer, or package managers

3. **Rust 1.85+**
   - Use the MSVC toolchain (default on Windows)
   - Install via [rustup](https://rustup.rs)

### Installation Methods

#### Option A: Using winget (Recommended)

```powershell
# Install Visual Studio Build Tools with C++ workload
winget install Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"

# Install CMake
winget install Kitware.CMake

# Install Rust
winget install Rustlang.Rustup
```

#### Option B: Using Chocolatey

```powershell
# Install chocolatey first if needed: https://chocolatey.org/install
choco install visualstudio2022buildtools --package-parameters "--add Microsoft.VisualStudio.Workload.VCTools"
choco install cmake --installargs 'ADD_CMAKE_TO_PATH=System'
choco install rustup.install
```

#### Option C: Manual Installation

1. Download [Visual Studio Build Tools 2022](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
   - Select "Desktop development with C++" workload during installation

2. Download [CMake](https://cmake.org/download/)
   - Select "Add CMake to the system PATH" during installation

3. Download [Rustup](https://rustup.rs)
   - Follow default installation prompts

### Verify Installation

Open a **new** terminal after installation and verify:

```powershell
# Check Rust
rustc --version   # Should show 1.85+
cargo --version

# Check CMake
cmake --version   # Should show 3.20+

# Check C++ compiler
cl              # Should show Microsoft C/C++ compiler info
```

If `cl` is not found, you may need to run builds from the "Developer Command Prompt for VS 2022" or run:
```powershell
# Initialize Visual Studio environment
& "C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
```

## Building

### Clone and Build

```powershell
git clone https://github.com/saorsa-labs/communitas.git
cd communitas

# Build headless binary
cargo build --release -p communitas-headless

# Output: target\release\communitas-headless.exe
```

### Common Build Commands

```powershell
# Build all workspace crates
cargo build --release

# Build and run tests
cargo test

# Build with verbose output (useful for debugging build issues)
cargo build --release -p communitas-headless -vv
```

## Known Issues

### `--all-targets` Fails on Windows

```
error: could not compile `libfuzzer-sys`
```

**Cause**: The `libfuzzer-sys` crate only supports Linux.

**Solution**: Don't use `--all-targets` flag. Instead:
```powershell
# This works:
cargo build --release

# This fails on Windows:
cargo build --release --all-targets  # Don't use this
```

### CMake Not Found

```
error: failed to run custom build command for `aws-lc-sys`
CMake Error: cmake version X.X or higher is required
```

**Solution**:
1. Ensure CMake is installed: `cmake --version`
2. Ensure CMake is in PATH
3. Try running from "Developer Command Prompt for VS 2022"

### MSVC Compiler Not Found

```
error: linker `link.exe` not found
```

**Solution**:
1. Install Visual Studio Build Tools with C++ workload
2. Run from "Developer Command Prompt for VS 2022"
3. Or initialize vcvars64.bat before building

### Long Path Issues

Windows has a 260-character path limit by default.

**Solution**: Enable long paths in Windows:
```powershell
# Run as Administrator
New-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" -Name "LongPathsEnabled" -Value 1 -PropertyType DWORD -Force
```

## Alternative: Pre-built Binaries

If you don't need to build from source, download pre-built binaries from [Releases](https://github.com/saorsa-labs/communitas/releases):

- `communitas-headless-windows-x86_64.zip` - Windows x64 binary

Extract and run directly - no build tools required.

## Why CMake is Required

Communitas uses `ant-quic` for QUIC networking, which depends on `aws-lc-rs` (AWS Libcrypto for Rust). AWS Libcrypto is a C library that requires CMake to compile.

**Why AWS-LC instead of pure-Rust alternatives like ring?**
- AWS Libcrypto is FIPS 140-3 validated (important for enterprise/government compliance)
- Maintained by AWS with regular security audits
- API-compatible with the ring crate
- Better performance in some benchmarks (AES-GCM on modern CPUs)

The tradeoff is that builds require CMake and a C compiler, but the security guarantees are worth it for a cryptographic networking application.

## Troubleshooting

### First Build is Slow

The first build compiles AWS Libcrypto from source, which takes 1-3 minutes. Subsequent builds are cached and much faster.

### Build Fails in Corporate Environment

Corporate networks may have proxy or firewall restrictions. Ensure:
- Access to crates.io (Rust packages)
- Access to github.com (source dependencies)
- Git configured with proxy if needed

### Need Help?

- Check [GitHub Issues](https://github.com/saorsa-labs/communitas/issues)
- Join the community Discord (if available)
- File a new issue with build logs
