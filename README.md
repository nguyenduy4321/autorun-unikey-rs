# autorun-unikey-rs

A lightning-fast, ultra-lightweight Windows startup script for Unikey, written in pure Rust.

## Features

- **Blazing Fast**: Directly uses Windows native APIs (Win32) instead of relying on `powershell` or `tasklist` / `taskkill`. Execution takes less than a millisecond.
- **Ultra Lightweight**: Optimized for size.
- **Ghost Layout Fix**: Automatically unloads the annoying Vietnamese ("vi") ghost keyboard layout (042A) on startup.
- **Silent Execution**: Runs completely in the background without flashing a CMD window.
- **Auto-Startup Management**: Removes Unikey's default registry autorun (which often causes the layout bug) and uses a native shortcut in the Startup folder instead.

## How it works

1. Removes Unikey's registry autorun to prevent race conditions.
2. Creates a startup shortcut (runs silently).
3. Launches `UnikeyNT.exe` if it isn't running.
4. Unloads the ghost Vietnamese keyboard layout using `LoadKeyboardLayoutA` & `UnloadKeyboardLayout`.

## Testing the Layout Fix

You can run `add-vi-layout.cmd` to forcefully add the Vietnamese ghost layout to your Windows system. 
Then, run `autorun-unikey.exe` and watch the layout instantly disappear!

## Build from Source

```bash
cargo build --release
```
The executable will be located in `target/release/autorun-unikey.exe`.

## Next Steps

- **Always run with Admin privileges without UAC popup**: Investigate methods like registering the executable as a Windows Service or using Windows Task Scheduler to run with highest privileges silently on startup.
