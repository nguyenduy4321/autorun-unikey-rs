#![windows_subsystem = "windows"]

use std::env;
use std::ffi::CStr;
use std::os::windows::process::CommandExt;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use winreg::enums::*;
use winreg::RegKey;

use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

const CREATE_NO_WINDOW: u32 = 0x08000000;

unsafe fn is_process_running(process_name: &str) -> bool {
    let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    if snapshot == INVALID_HANDLE_VALUE {
        return false;
    }

    let mut entry: PROCESSENTRY32 = std::mem::zeroed();
    entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

    let mut found = false;
    if Process32First(snapshot, &mut entry) != 0 {
        loop {
            let exe_name = CStr::from_ptr(entry.szExeFile.as_ptr() as *const i8).to_string_lossy();
            if exe_name.eq_ignore_ascii_case(process_name) {
                found = true;
                break;
            }
            if Process32Next(snapshot, &mut entry) == 0 {
                break;
            }
        }
    }

    CloseHandle(snapshot);
    found
}

unsafe fn find_and_kill_process(process_name: &str) {
    let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    if snapshot == INVALID_HANDLE_VALUE {
        return;
    }

    let mut entry: PROCESSENTRY32 = std::mem::zeroed();
    entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

    if Process32First(snapshot, &mut entry) != 0 {
        loop {
            let exe_name = CStr::from_ptr(entry.szExeFile.as_ptr() as *const i8).to_string_lossy();
            if exe_name.eq_ignore_ascii_case(process_name) {
                let process_handle = OpenProcess(PROCESS_TERMINATE, 0, entry.th32ProcessID);
                if process_handle != 0 {
                    TerminateProcess(process_handle, 1);
                    CloseHandle(process_handle);
                }
            }
            if Process32Next(snapshot, &mut entry) == 0 {
                break;
            }
        }
    }

    CloseHandle(snapshot);
}

fn remove_unikey_autorun() {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(run_key) =
        hkcu.open_subkey_with_flags("Software\\Microsoft\\Windows\\CurrentVersion\\Run", KEY_READ | KEY_WRITE)
    {
        if run_key.get_value::<String, _>("Unikey").is_ok() {
            unsafe { find_and_kill_process("UnikeyNT.exe") };
            let _ = run_key.delete_value("Unikey");
        }
    }
}

fn remove_old_shortcut() {
    if let Ok(appdata) = env::var("APPDATA") {
        let startup_folder =
            Path::new(&appdata).join("Microsoft\\Windows\\Start Menu\\Programs\\Startup");
        let current_exe = env::current_exe().unwrap_or_else(|_| PathBuf::from("autorun-unikey.exe"));

        let shortcut_name = current_exe
            .file_stem()
            .unwrap_or_else(|| std::ffi::OsStr::new("autorun-unikey"))
            .to_string_lossy();
        let shortcut_path = startup_folder.join(format!("{}.lnk", shortcut_name));

        if shortcut_path.exists() {
            let _ = std::fs::remove_file(shortcut_path);
        }
    }
}

fn setup_task_scheduler() {
    let current_exe = env::current_exe().unwrap_or_else(|_| PathBuf::from("autorun-unikey.exe"));
    let task_name = "AutorunUnikeyRS";

    let check = Command::new("schtasks")
        .args(&["/query", "/tn", task_name])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    let task_exists = if let Ok(output) = check {
        output.status.success()
    } else {
        false
    };

    if !task_exists {
        let create = Command::new("schtasks")
            .args(&[
                "/create",
                "/tn", task_name,
                "/tr", current_exe.to_str().unwrap(),
                "/sc", "onlogon",
                "/rl", "highest",
                "/f"
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .status();

        if let Ok(status) = create {
            if !status.success() {
                // If it fails (likely due to missing admin rights), elevate!
                unsafe {
                    let mut path_null = current_exe.to_string_lossy().into_owned();
                    path_null.push('\0');
                    windows_sys::Win32::UI::Shell::ShellExecuteA(
                        0,
                        b"runas\0".as_ptr(),
                        path_null.as_ptr(),
                        std::ptr::null(),
                        std::ptr::null(),
                        windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOW,
                    );
                }
                // Exit current non-admin process
                std::process::exit(0);
            }
        }
    }
}

fn remove_ghost_layout() {
    #[link(name = "user32")]
    extern "system" {
        fn LoadKeyboardLayoutA(pwszklid: *const u8, flags: u32) -> isize;
        fn UnloadKeyboardLayout(hkl: isize) -> i32;
    }

    unsafe {
        // 0000042A là mã của bàn phím tiếng Việt (Vietnamese)
        // KLF_ACTIVATE = 1
        let hkl = LoadKeyboardLayoutA(b"0000042A\0".as_ptr(), 1);
        if hkl != 0 {
            UnloadKeyboardLayout(hkl);
        }
    }
}

fn show_message(msg: &str, title: &str) {
    let mut msg_w: Vec<u16> = std::ffi::OsStr::new(msg).encode_wide().collect();
    msg_w.push(0);
    let mut title_w: Vec<u16> = std::ffi::OsStr::new(title).encode_wide().collect();
    title_w.push(0);
    unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxW(
            0,
            msg_w.as_ptr(),
            title_w.as_ptr(),
            windows_sys::Win32::UI::WindowsAndMessaging::MB_OK | windows_sys::Win32::UI::WindowsAndMessaging::MB_ICONINFORMATION,
        );
    }
}

fn ensure_unikey_running() {
    let unikey_exe = "UnikeyNT.exe";
    let is_running = unsafe { is_process_running(unikey_exe) };

    if !is_running {
        let mut unikey_path = env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
        unikey_path.pop();
        unikey_path.push(unikey_exe);

        if unikey_path.exists() {
            let _ = Command::new(&unikey_path)
                .current_dir(unikey_path.parent().unwrap())
                .spawn();
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let cmd = args[1].as_str();
        match cmd {
            "--uninstall" => {
                let task_name = "AutorunUnikeyRS";
                let _ = Command::new("schtasks")
                    .args(&["/delete", "/tn", task_name, "/f"])
                    .creation_flags(CREATE_NO_WINDOW)
                    .status();
                remove_unikey_autorun();
                remove_old_shortcut();
                show_message("Uninstall completed! All Autorun entries and Task Scheduler tasks have been removed.", "Autorun Unikey RS");
                return;
            }
            "--demo-mode" => {
                // Thêm layout để demo
                #[link(name = "user32")]
                extern "system" {
                    fn LoadKeyboardLayoutA(pwszklid: *const u8, flags: u32) -> isize;
                }
                unsafe {
                    LoadKeyboardLayoutA(b"0000042A\0".as_ptr(), 1);
                }

                show_message(
                    "Bạn có đang bị dính bàn phím tiếng Việt (VIE) thừa thãi ở góc màn hình như thế này không?", 
                    "Demo - Bước 1"
                );
                
                remove_ghost_layout();

                show_message(
                    "Đã xóa xong! Từ nay phần mềm sẽ tự động dọn dẹp lỗi này mỗi khi bạn khởi động máy.", 
                    "Demo - Bước 2"
                );
                return;
            }
            _ => {}
        }
    }

    // 1. Turn off "Auto-run Unikey at boot time" of UnikeyNT natively via Registry
    remove_unikey_autorun();

    // 2. Remove old startup shortcut if it exists
    remove_old_shortcut();

    // 3. Register as a Task Scheduler task to run as Admin without UAC silently
    setup_task_scheduler();

    // 4. Start UnikeyNT
    ensure_unikey_running();

    // 5. Xóa lỗi bàn phím tiếng Việt (Ghost layout) bằng Win32 API gốc (siêu nhanh)
    remove_ghost_layout();
}
