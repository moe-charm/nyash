# 🪟 Windows Distribution Guide

Step-by-step guide to creating Windows distributions of Nyash applications.

## 📋 Prerequisites

### For Cross-Compilation from Linux/WSL
- Rust toolchain
- cargo-xwin: `cargo install cargo-xwin`

### For Native Windows Build
- Rust (MSVC toolchain)
- Visual Studio Build Tools (C++ Desktop Development)
- LLVM/clang (optional, for AOT)

## 🏗️ Building for Windows

### Option 1: Cross-Compile from WSL (Recommended)

```bash
# One-time setup
cargo install cargo-xwin

# Build main executable
cargo xwin build --target x86_64-pc-windows-msvc --release

# Build plugins
cd plugins/nyash-filebox-plugin
cargo xwin build --target x86_64-pc-windows-msvc --release
```

### Option 2: Native Windows Build

```powershell
# On Windows with MSVC
cargo build --release

# Plugins
cd plugins\nyash-filebox-plugin
cargo build --release
```

## 📦 Creating a Distribution Package

### Recommended Directory Structure

```
my-nyash-app/
├── nyash.exe           # Main executable (4.1MB)
├── nyash.toml          # Configuration
├── app.nyash           # Your application
├── README.txt          # User instructions
└── plugins/            # Plugin DLLs
    ├── nyash_array_plugin.dll
    ├── nyash_filebox_plugin.dll
    ├── nyash_map_plugin.dll
    └── ...
```

### Step-by-Step Creation

#### 1. Create Distribution Directory

```bash
# From WSL or Linux
mkdir -p dist/my-nyash-app/plugins

# Or from Windows
mkdir dist\my-nyash-app\plugins
```

#### 2. Copy Main Executable

```bash
cp target/x86_64-pc-windows-msvc/release/nyash.exe dist/my-nyash-app/
```

#### 3. Create Minimal nyash.toml

```toml
# dist/my-nyash-app/nyash.toml
[plugin_paths]
search_paths = ["./plugins"]

[libraries]
# Use .so extension - it works on Windows too!
[libraries."libnyash_filebox_plugin.so"]
boxes = ["FileBox"]
path = "libnyash_filebox_plugin.so"

[libraries."libnyash_string_plugin.so"]
boxes = ["StringBox"]
path = "libnyash_string_plugin.so"

# Add other plugins as needed...
```

#### 4. Copy Plugin DLLs

```bash
# Note: Windows removes 'lib' prefix from plugin names
cp plugins/nyash-filebox-plugin/target/x86_64-pc-windows-msvc/release/nyash_filebox_plugin.dll dist/my-nyash-app/plugins/
cp plugins/nyash-array-plugin/target/x86_64-pc-windows-msvc/release/nyash_array_plugin.dll dist/my-nyash-app/plugins/
# ... repeat for other plugins
```

#### 5. Add Your Application

```bash
cp your_app.nyash dist/my-nyash-app/app.nyash
```

#### 6. Create Run Script (Optional)

Create `run.bat`:
```batch
@echo off
nyash.exe app.nyash %*
```

## 🚀 Real-World Example

Here's the exact process used on 2025-08-29:

```bash
# 1. Build everything for Windows
cargo xwin build --target x86_64-pc-windows-msvc --release -j32

# 2. Build plugins
for plugin in filebox array map string integer; do
  (cd plugins/nyash-$plugin-plugin && \
   cargo xwin build --target x86_64-pc-windows-msvc --release --quiet)
done

# 3. Create distribution
mkdir -p /mnt/c/tmp/nyash-windows-dist/plugins
cp target/x86_64-pc-windows-msvc/release/nyash.exe /mnt/c/tmp/nyash-windows-dist/
cp nyash.toml /mnt/c/tmp/nyash-windows-dist/  # Modified version

# 4. Copy plugins
for plugin in filebox array map string integer; do
  cp plugins/nyash-$plugin-plugin/target/x86_64-pc-windows-msvc/release/nyash_${plugin}_plugin.dll \
     /mnt/c/tmp/nyash-windows-dist/plugins/
done

# 5. Test on Windows
cmd.exe /c "cd C:\tmp\nyash-windows-dist && nyash.exe test.nyash"
```

## 🎯 Testing Your Distribution

### Basic Test Program

Create `test.nyash`:
```nyash
print("=== Testing Nyash Distribution ===")

// Test plugins
local str = new StringBox()
local arr = new ArrayBox()
local map = new MapBox()

arr.push("Distribution")
arr.push("works!")

map.set("status", "success")

print("Plugins loaded: ✅")
print("Array test: " + arr.get(0) + " " + arr.get(1))
print("Map test: " + map.get("status"))
```

### Running Tests

```batch
:: Basic run
nyash.exe test.nyash

:: With verbose output
nyash.exe --verbose test.nyash

:: Check plugin loading
set NYASH_CLI_VERBOSE=1
nyash.exe test.nyash
```

## 📝 Important Notes

### Plugin Name Differences

| Platform | Library Name | Note |
|----------|--------------|------|
| Linux | `libnyash_example_plugin.so` | With 'lib' prefix |
| Windows | `nyash_example_plugin.dll` | No 'lib' prefix! |
| macOS | `libnyash_example_plugin.dylib` | With 'lib' prefix |

The plugin loader handles this automatically!

### Path Separators

Use forward slashes in nyash.toml - they work on Windows:
```toml
search_paths = ["./plugins"]  # Works on Windows!
```

### Dependencies

Most Nyash distributions are self-contained. However, some plugins might need:
- Visual C++ Redistributables (usually already installed)
- Python DLL (for Python plugin)

## 🔐 Code Signing (Optional)

For distribution without security warnings:

```powershell
# Sign the executable
signtool sign /a /t http://timestamp.digicert.com nyash.exe

# Sign all DLLs
Get-ChildItem -Path plugins -Filter *.dll | ForEach-Object {
    signtool sign /a /t http://timestamp.digicert.com $_.FullName
}
```

## 📦 Creating an Installer

### Using Inno Setup

```ini
[Setup]
AppName=My Nyash Application
AppVersion=1.0
DefaultDirName={pf}\MyNyashApp
DefaultGroupName=My Nyash App

[Files]
Source: "nyash.exe"; DestDir: "{app}"
Source: "nyash.toml"; DestDir: "{app}"
Source: "app.nyash"; DestDir: "{app}"
Source: "plugins\*.dll"; DestDir: "{app}\plugins"

[Icons]
Name: "{group}\My Nyash App"; Filename: "{app}\nyash.exe"; Parameters: "app.nyash"
```

### Using zip for Simple Distribution

```bash
# From WSL/Linux
cd dist
zip -r my-nyash-app-windows.zip my-nyash-app/

# Or use 7-Zip on Windows
7z a my-nyash-app-windows.zip my-nyash-app\
```

## ✅ Distribution Checklist

- [ ] Build nyash.exe for Windows
- [ ] Build all required plugin DLLs
- [ ] Create platform-agnostic nyash.toml
- [ ] Copy all files to distribution folder
- [ ] Test on a clean Windows system
- [ ] Create README with instructions
- [ ] Package as zip or installer
- [ ] (Optional) Code sign binaries

## 🆘 Troubleshooting

### "Plugin not found" errors
1. Check plugin DLL exists in `plugins/` folder
2. Verify naming (no 'lib' prefix on Windows)
3. Run with `--verbose` for detailed logs

### Missing DLL errors
- Install Visual C++ Redistributables
- Check plugin dependencies with `dumpbin /dependents plugin.dll`

### Performance issues
- Ensure release build (`--release` flag)
- Disable Windows Defender real-time scanning for Nyash folder

## 📚 See Also

- [Cross-Platform Guide](cross-platform.md)
- [Plugin Development](../../reference/plugin-system/)
- [Build Overview](README.md)