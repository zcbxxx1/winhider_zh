/*
 * =============================================================================
 * WinHider CLI - Headless Window Management Tool
 * =============================================================================
 *
 * Filename: main.rs
 * Author: bigwiz
 * Description: Command-line interface for WinHider, providing window visibility
 *              control without GUI dependencies.
 *
 * Features:
 * - List all visible windows
 * - Hide/unhide windows from screen capture
 * - Hide/unhide windows from taskbar/task switcher
 * - Show detailed window information
 * - Interactive command mode
 *
 * Commands:
 * - list: List all visible windows
 * - hide <window_id>: Hide a window from screen capture
 * - unhide <window_id>: Unhide a window from screen capture
 * - hidetask <window_id>: Hide a window from taskbar/task switcher
 * - unhidetask <window_id>: Unhide a window from taskbar/task switcher
 * - info <window_id>: Show detailed information about a window
 * - help: Show help menu
 * - exit: Exit the application
 *
 * Designed At - Bitmutex Technologies
 * =============================================================================
 */

use clap::{Parser, Subcommand};
use std::ffi::c_void;
use std::time::{SystemTime, UNIX_EPOCH};
use windows::core::{s};
use windows::Win32::Foundation::*;
use windows::Win32::System::Diagnostics::Debug::*;
use windows::Win32::System::LibraryLoader::*;
use windows::Win32::System::Memory::*;
use windows::Win32::System::Threading::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::System::Diagnostics::ToolHelp::*;
use windows::Win32::Graphics::Gdi::*;

// ===============================
// CONSTANTS & CONFIG
// ===============================

const APP_NAME: &str = "WinHider CLI";

// Windows to ignore in the list
const IGNORED_WINDOWS: &[&str] = &[
    "Program Manager",
    "Settings",
    "Microsoft Text Input Application",
    "WinHider"
];

// ===============================
// Data Models
// ===============================

#[derive(Clone, Debug)]
struct WindowInfo {
    pub hwnd: HWND,
    pub pid: u32,
    pub title: String,
    pub class_name: String,
    pub process_name: String,
    pub is_visible: bool,
}

enum InjectionAction {
    HideCapture,
    ShowCapture,
    HideTaskbar,
    ShowTaskbar,
}

// ===============================
// CLI Commands
// ===============================

#[derive(Parser)]
#[command(name = APP_NAME)]
#[command(version = "1.0.1")]
#[command(about = "Headless window visibility controller - runs in interactive mode by default")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// List all visible windows
    List,
    /// Hide a window from screen capture
    Hide { window_id: String },
    /// Unhide a window from screen capture
    Unhide { window_id: String },
    /// Hide a window from taskbar/task switcher
    Hidetask { window_id: String },
    /// Unhide a window from taskbar/task switcher
    Unhidetask { window_id: String },
    /// Show detailed information about a window
    Info { window_id: String },
    /// Show help menu
    Help,
    /// Interactive mode
    Interactive,
}

// ===============================
// CORE FUNCTIONS
// ===============================

fn enumerate_windows() -> Vec<WindowInfo> {
    let mut list = Vec::new();

    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        unsafe {
            if !IsWindowVisible(hwnd).as_bool() { return BOOL(1); }

            let mut title_buf = [0u16; 256];
            let title_len = GetWindowTextW(hwnd, &mut title_buf);
            if title_len == 0 { return BOOL(1); }

            let title = String::from_utf16_lossy(&title_buf[..title_len as usize]);
            if IGNORED_WINDOWS.contains(&title.as_str()) { return BOOL(1); }

            let mut class_buf = [0u16; 256];
            let class_len = GetClassNameW(hwnd, &mut class_buf);
            let class_name = String::from_utf16_lossy(&class_buf[..class_len as usize]);

            let mut pid = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));

            let process_name = get_process_name(pid);

            let list = &mut *(lparam.0 as *mut Vec<WindowInfo>);
            list.push(WindowInfo {
                hwnd,
                pid,
                title,
                class_name,
                process_name,
                is_visible: IsWindowVisible(hwnd).as_bool(),
            });
            BOOL(1)
        }
    }

    unsafe { let _ = EnumWindows(Some(enum_proc), LPARAM(&mut list as *mut _ as isize)); }
    list
}

fn get_window_by_id(id_str: &str) -> Option<WindowInfo> {
    let windows = enumerate_windows();

    // Try parsing as PID first
    if let Ok(pid) = id_str.parse::<u32>() {
        if let Some(window) = windows.iter().find(|w| w.pid == pid) {
            return Some(window.clone());
        }
    }

    // Try matching by process name (with or without .exe extension)
    let process_name_lower = id_str.to_lowercase();
    let process_name_no_ext = if process_name_lower.ends_with(".exe") {
        process_name_lower.trim_end_matches(".exe")
    } else {
        &process_name_lower
    };

    // First try exact match with .exe
    if let Some(window) = windows.iter().find(|w| w.process_name.to_lowercase() == process_name_lower) {
        return Some(window.clone());
    }

    // Then try exact match without .exe
    if let Some(window) = windows.iter().find(|w| {
        let w_name_lower = w.process_name.to_lowercase();
        let w_name_no_ext = if w_name_lower.ends_with(".exe") {
            w_name_lower.trim_end_matches(".exe")
        } else {
            &w_name_lower
        };
        w_name_no_ext == process_name_no_ext
    }) {
        return Some(window.clone());
    }

    // Try partial match by process name
    if let Some(window) = windows.iter().find(|w| w.process_name.to_lowercase().contains(process_name_no_ext)) {
        return Some(window.clone());
    }

    // Fallback: Try parsing as index (for backward compatibility)
    if let Ok(index) = id_str.parse::<usize>() {
        if index > 0 && index <= windows.len() {
            return Some(windows[index - 1].clone());
        }
    }

    // Fallback: Try parsing as HWND value
    if let Ok(hwnd_val) = id_str.parse::<isize>() {
        let target_hwnd = HWND(hwnd_val);
        return windows.into_iter().find(|w| w.hwnd == target_hwnd);
    }

    // Fallback: Try matching by title (partial)
    windows.into_iter().find(|w| w.title.to_lowercase().contains(&id_str.to_lowercase()))
}

fn inject_payload(target_pid: u32, action: InjectionAction) -> std::result::Result<String, String> {
    unsafe {
        let mut master_dll_path = std::env::current_exe()
            .map_err(|e| e.to_string())?
            .parent()
            .unwrap()
            .join("winhider_payload.dll");

        if !master_dll_path.exists() {
             if let Ok(cwd) = std::env::current_dir() {
                 master_dll_path = cwd.join("target").join("release").join("winhider_payload.dll");
             }
        }
        if !master_dll_path.exists() { return Err("Base DLL not found. Make sure winhider_payload.dll is in the same directory.".to_string()); }

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let keyword = match action {
            InjectionAction::HideCapture => "hidecapture",
            InjectionAction::ShowCapture => "showcapture",
            InjectionAction::HideTaskbar => "hidetaskbar",
            InjectionAction::ShowTaskbar => "showtaskbar",
        };

        let new_filename = format!("winhider_payload_{}_{}.dll", keyword, timestamp);
        let target_dll_path = master_dll_path.parent().unwrap().join(&new_filename);

        if let Err(e) = std::fs::copy(&master_dll_path, &target_dll_path) {
            return Err(format!("Failed to create temp DLL: {}", e));
        }

        let path_str = target_dll_path.to_str().unwrap();
        let mut path_bytes: Vec<u8> = path_str.bytes().collect();
        path_bytes.push(0);

        let process = OpenProcess(
            PROCESS_CREATE_THREAD | PROCESS_QUERY_INFORMATION | PROCESS_VM_OPERATION | PROCESS_VM_WRITE | PROCESS_VM_READ,
            false,
            target_pid
        ).map_err(|e| format!("OpenProcess failed: {}", e))?;

        let remote_mem = VirtualAllocEx(process, None, path_bytes.len(), MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
        if remote_mem.is_null() { let _ = CloseHandle(process); return Err("Memory allocation failed".to_string()); }

        let mut written = 0;
        let write_res = WriteProcessMemory(process, remote_mem, path_bytes.as_ptr() as *const c_void, path_bytes.len(), Some(&mut written));
        if write_res.is_err() { let _ = VirtualFreeEx(process, remote_mem, 0, MEM_RELEASE); let _ = CloseHandle(process); return Err("WriteProcessMemory failed".to_string()); }

        let kernel32 = GetModuleHandleA(s!("kernel32.dll")).unwrap();
        let load_lib = GetProcAddress(kernel32, s!("LoadLibraryA"));

        if load_lib.is_none() { let _ = VirtualFreeEx(process, remote_mem, 0, MEM_RELEASE); let _ = CloseHandle(process); return Err("GetProcAddress failed".to_string()); }

        let start_routine = std::mem::transmute::<unsafe extern "system" fn() -> isize, unsafe extern "system" fn(*mut c_void) -> u32>(std::mem::transmute(load_lib));

        let thread = CreateRemoteThread(process, None, 0, Some(start_routine), Some(remote_mem), 0, None)
            .map_err(|e| format!("CreateRemoteThread failed: {}", e))?;

        WaitForSingleObject(thread, 2000);
        let _ = VirtualFreeEx(process, remote_mem, 0, MEM_RELEASE);
        let _ = CloseHandle(thread);
        let _ = CloseHandle(process);

        Ok(new_filename)
    }
}

fn clean_temp_files() {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with("winhider_payload_") && name.ends_with(".dll") {
                            let _ = std::fs::remove_file(path);
                        }
                    }
                }
            }
        }
    }
}

fn get_window_details(hwnd: HWND) -> Option<(String, String, RECT, u32)> {
    unsafe {
        let mut title_buf = [0u16; 256];
        let title_len = GetWindowTextW(hwnd, &mut title_buf);
        let title = String::from_utf16_lossy(&title_buf[..title_len as usize]);

        let mut class_buf = [0u16; 256];
        let class_len = GetClassNameW(hwnd, &mut class_buf);
        let class_name = String::from_utf16_lossy(&class_buf[..class_len as usize]);

        let mut rect = RECT::default();
        let _ = GetWindowRect(hwnd, &mut rect);

        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));

        Some((title, class_name, rect, pid))
    }
}

fn get_process_name(pid: u32) -> String {
    unsafe {
        let process = match OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid) {
            Ok(p) => p,
            Err(_) => return "unknown".to_string(),
        };

        let mut exe_buf = [0u16; 1024];
        let mut size = exe_buf.len() as u32;
        
        if QueryFullProcessImageNameW(process, PROCESS_NAME_FORMAT(0), windows::core::PWSTR(exe_buf.as_mut_ptr()), &mut size).is_ok() {
            let exe_path = String::from_utf16_lossy(&exe_buf[..size as usize]);
            if let Some(exe_name) = std::path::Path::new(&exe_path).file_name() {
                exe_name.to_string_lossy().to_string()
            } else {
                "unknown".to_string()
            }
        } else {
            "unknown".to_string()
        }
    }
}

// ===============================
// COMMAND HANDLERS
// ===============================

fn handle_list() {
    let windows = enumerate_windows();
    if windows.is_empty() {
        println!("No visible windows found.");
        return;
    }

    println!("{:<5} {:<10} {:<20} {:<30} {}", "ID", "PID", "Process", "Title", "Class");
    println!("{}", "=".repeat(100));

    for (i, window) in windows.iter().enumerate() {
        let truncated_process = if window.process_name.len() > 18 {
            format!("{}...", &window.process_name[..15])
        } else {
            window.process_name.clone()
        };

        let truncated_title = if window.title.len() > 28 {
            format!("{}...", &window.title[..25])
        } else {
            window.title.clone()
        };

        let truncated_class = if window.class_name.len() > 15 {
            format!("{}...", &window.class_name[..12])
        } else {
            window.class_name.clone()
        };

        println!("{:<5} {:<10} {:<20} {:<30} {}",
                 i + 1,
                 window.pid,
                 truncated_process,
                 truncated_title,
                 truncated_class);
    }
    println!("\nTotal: {} windows", windows.len());
}

fn handle_hide(window_id: &str) {
    match get_window_by_id(window_id) {
        Some(window) => {
            match inject_payload(window.pid, InjectionAction::HideCapture) {
                Ok(_) => println!("Successfully hid window '{}' from screen capture.", window.title),
                Err(e) => println!("Error hiding window: {}", e),
            }
        }
        None => println!("Window '{}' not found. Use 'list' to see available windows.", window_id),
    }
}

fn handle_unhide(window_id: &str) {
    match get_window_by_id(window_id) {
        Some(window) => {
            match inject_payload(window.pid, InjectionAction::ShowCapture) {
                Ok(_) => println!("Successfully unhid window '{}' from screen capture.", window.title),
                Err(e) => println!("Error unhiding window: {}", e),
            }
        }
        None => println!("Window '{}' not found. Use 'list' to see available windows.", window_id),
    }
}

fn handle_hidetask(window_id: &str) {
    match get_window_by_id(window_id) {
        Some(window) => {
            match inject_payload(window.pid, InjectionAction::HideTaskbar) {
                Ok(_) => println!("Successfully hid window '{}' from taskbar.", window.title),
                Err(e) => println!("Error hiding window from taskbar: {}", e),
            }
        }
        None => println!("Window '{}' not found. Use 'list' to see available windows.", window_id),
    }
}

fn handle_unhidetask(window_id: &str) {
    match get_window_by_id(window_id) {
        Some(window) => {
            match inject_payload(window.pid, InjectionAction::ShowTaskbar) {
                Ok(_) => println!("Successfully unhid window '{}' from taskbar.", window.title),
                Err(e) => println!("Error unhiding window from taskbar: {}", e),
            }
        }
        None => println!("Window '{}' not found. Use 'list' to see available windows.", window_id),
    }
}

fn handle_info(window_id: &str) {
    match get_window_by_id(window_id) {
        Some(window) => {
            if let Some((title, class_name, rect, pid)) = get_window_details(window.hwnd) {
                println!("Window Information:");
                println!("==================");
                println!("HWND: {:?}", window.hwnd);
                println!("PID: {}", pid);
                println!("Title: {}", title);
                println!("Class: {}", class_name);
                println!("Position: ({}, {})", rect.left, rect.top);
                println!("Size: {}x{}", rect.right - rect.left, rect.bottom - rect.top);
                println!("Visible: {}", window.is_visible);

                // Get process name
                unsafe {
                    let process = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid);
                    if let Ok(process) = process {
                        // Note: GetModuleFileNameExW is not available in this version, skipping process name
                        let _ = CloseHandle(process);
                    }
                }
            }
        }
        None => println!("Window '{}' not found. Use 'list' to see available windows.", window_id),
    }
}

fn handle_help() {
    println!("{} v{}", APP_NAME, env!("CARGO_PKG_VERSION"));
    println!("Headless window visibility controller");
    println!();
    println!("COMMANDS:");
    println!("  list                    List all visible windows");
    println!("  hide <window_id>        Hide a window from screen capture");
    println!("  unhide <window_id>      Unhide a window from screen capture");
    println!("  hidetask <window_id>    Hide a window from taskbar/task switcher");
    println!("  unhidetask <window_id>  Unhide a window from taskbar/task switcher");
    println!("  info <window_id>        Show detailed information about a window");
    println!("  help                    Show this help menu");
    println!("  interactive             Enter interactive mode");
    println!("  exit                    Exit the application");
    println!();
    println!("WINDOW IDENTIFIERS:");
    println!("  - Process ID (PID) number");
    println!("  - Process name (with or without .exe extension, partial match allowed)");
    println!("  - Index number from 'list' command (1-based, fallback)");
    println!("  - HWND value (as integer, fallback)");
    println!("  - Window title (partial match, case-insensitive, fallback)");
    println!();
    println!("EXAMPLES:");
    println!("  winhider-cli list");
    println!("  winhider-cli hide notepad");
    println!("  winhider-cli hide notepad.exe");
    println!("  winhider-cli hide 1234");
    println!("  winhider-cli info chrome");
}

fn print_ascii_art() {
    let art = r#"
██╗    ██╗██╗███╗   ██╗██╗  ██╗██╗██████╗ ███████╗██████╗
██║    ██║██║████╗  ██║██║  ██║██║██╔══██╗██╔════╝██╔══██╗
██║ █╗ ██║██║██╔██╗ ██║███████║██║██║  ██║█████╗  ██████╔╝
██║███╗██║██║██║╚██╗██║██╔══██║██║██║  ██║██╔══╝  ██╔══██╗
╚███╔███╔╝██║██║ ╚████║██║  ██║██║██████╔╝███████╗██║  ██║
 ╚══╝╚══╝ ╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝╚═╝╚═════╝ ╚══════╝╚═╝  ╚═╝
"#;

    let lines: Vec<&str> = art.lines().collect();
    let colors = [
        "\x1b[38;5;39m",  // Blue
        "\x1b[38;5;45m",  // Cyan
        "\x1b[38;5;51m",  // Light Blue
        "\x1b[38;5;87m",  // Light Cyan
        "\x1b[38;5;93m",  // Light Purple
        "\x1b[38;5;99m",  // Purple
    ];
    let reset = "\x1b[0m";

    for (i, line) in lines.iter().enumerate() {
        if !line.trim().is_empty() {
            let color = colors[i % colors.len()];
            println!("{}{}{}", color, line, reset);
        }
    }
}

fn run_interactive() {
    print_ascii_art();
    println!("{} - Interactive Mode", APP_NAME);
    println!("Type 'help' for commands or 'exit' to quit.");
    println!();

    loop {
        print!("> ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parts[0].to_lowercase();

        match command.as_str() {
            "list" => handle_list(),
            "hide" => {
                if parts.len() < 2 {
                    println!("Usage: hide <window_id>");
                } else {
                    handle_hide(parts[1]);
                }
            }
            "unhide" => {
                if parts.len() < 2 {
                    println!("Usage: unhide <window_id>");
                } else {
                    handle_unhide(parts[1]);
                }
            }
            "hidetask" => {
                if parts.len() < 2 {
                    println!("Usage: hidetask <window_id>");
                } else {
                    handle_hidetask(parts[1]);
                }
            }
            "unhidetask" => {
                if parts.len() < 2 {
                    println!("Usage: unhidetask <window_id>");
                } else {
                    handle_unhidetask(parts[1]);
                }
            }
            "info" => {
                if parts.len() < 2 {
                    println!("Usage: info <window_id>");
                } else {
                    handle_info(parts[1]);
                }
            }
            "help" | "?" => handle_help(),
            "exit" | "quit" | "q" => break,
            _ => println!("Unknown command: {}. Type 'help' for available commands.", command),
        }

        println!();
    }
}

// ===============================
// MAIN
// ===============================

fn main() {
    clean_temp_files();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::List) => handle_list(),
        Some(Commands::Hide { window_id }) => handle_hide(&window_id),
        Some(Commands::Unhide { window_id }) => handle_unhide(&window_id),
        Some(Commands::Hidetask { window_id }) => handle_hidetask(&window_id),
        Some(Commands::Unhidetask { window_id }) => handle_unhidetask(&window_id),
        Some(Commands::Info { window_id }) => handle_info(&window_id),
        Some(Commands::Help) => handle_help(),
        Some(Commands::Interactive) => run_interactive(),
        None => run_interactive(),
    }
}