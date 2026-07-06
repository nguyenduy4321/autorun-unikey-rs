# <img src="icon_final.png" width="48" align="center" alt="icon"> autorun-unikey-rs

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

## Usage

> **[IMPORTANT]** You MUST place `autorun-unikey.exe` into the exact same folder as your `UniKeyNT.exe` before running it.

1. Download `autorun-unikey.exe` from the Releases page.
2. Place it in the folder where your `UniKeyNT.exe` is located.
3. Run `autorun-unikey.exe` manually one time.
4. (Optional) Windows will prompt you with a UAC window (Run as Administrator) to grant permission for the first-time setup.

## Interactive Demo Mode

Want to see the magic in action? You can run the executable via command line (CMD or PowerShell) to trigger an interactive tutorial:
```bash
autorun-unikey.exe --demo-mode
```
This mode will step-by-step start Unikey, ask you to look at your taskbar, and then instantly obliterate the Vietnamese ghost layout before your eyes!

## Uninstall

To safely unregister the Task Scheduler and remove all traces, run:
```bash
autorun-unikey.exe --uninstall
```

## Build from Source

```bash
cargo build --release
```
The executable will be located in `target/release/autorun-unikey.exe`.

