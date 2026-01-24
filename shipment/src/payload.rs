/*
 * =============================================================================
 * WinHider Payload DLL - Window Manipulation Library
 * =============================================================================
 *
 * Filename: payload.rs
 * Author: bigwiz
 * Description: Dynamic link library (DLL) payload shipment for WinHider that performs
 *              low-level window manipulation operations. Injected into target
 *              processes to hide/show windows from screen capture and taskbar.
 *
 * Features:
 * - DLL injection mechanism
 * - Window style manipulation
 * - Extended window style control
 * - Process-specific operations
 * - Thread-safe execution
 *
 * Technical Details:
 * - Uses Windows API for window manipulation
 * - Implements DllMain entry point
 * - Supports multiple injection actions
 * - Error handling and logging
 *
 * Created: 2024
 * License: Proprietary - Bitmutex Technologies
 * =============================================================================
 */

use windows::Win32::Foundation::*;
use windows::Win32::System::SystemServices::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows::Win32::System::Threading::GetCurrentProcessId;

static mut DLL_INSTANCE: HINSTANCE = HINSTANCE(0);

#[unsafe(no_mangle)]
#[allow(non_snake_case, unused_variables)]
pub extern "system" fn DllMain(
    dll_module: HINSTANCE,
    call_reason: u32,
    reserved: *mut std::ffi::c_void,
) -> BOOL {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            unsafe { DLL_INSTANCE = dll_module; }
            std::thread::spawn(|| {
                unsafe { apply_stealth(); }
            });
        }
        _ => {}
    }
    BOOL(1)
}

unsafe fn apply_stealth() {
    // 1. Parse Filename to determine actions
    let mut path_buffer = [0u16; 1024];
    let len = GetModuleFileNameW(DLL_INSTANCE, &mut path_buffer);
    let full_path = String::from_utf16_lossy(&path_buffer[..len as usize]).to_lowercase();
    
    // We use a bitmask to pass instructions to the enumeration callback
    // Bit 0: Hide Capture
    // Bit 1: Show Capture
    // Bit 2: Hide Taskbar
    // Bit 3: Show Taskbar
    let mut action_mask: isize = 0;

    if full_path.contains("hidecapture") { action_mask |= 1; }
    if full_path.contains("showcapture") { action_mask |= 2; }
    if full_path.contains("hidetaskbar") { action_mask |= 4; }
    if full_path.contains("showtaskbar") { action_mask |= 8; }

    let current_pid = GetCurrentProcessId();
    EnumWindows(Some(enum_window_proc), LPARAM(action_mask));
}

unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let mask = lparam.0;
    let mut window_pid = 0;
    let current_pid = GetCurrentProcessId();
    GetWindowThreadProcessId(hwnd, Some(&mut window_pid));

    // Only modify windows belonging to THIS process
    if window_pid == current_pid && IsWindowVisible(hwnd).as_bool() {
        
        // --- 1. Screen Capture Protection ---
        if (mask & 1) != 0 { 
            let _ = SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE); 
        }
        if (mask & 2) != 0 { 
            let _ = SetWindowDisplayAffinity(hwnd, WDA_NONE); 
        }

        // --- 2. Taskbar / Alt-Tab Visibility ---
        // To hide from Taskbar: Remove APPWINDOW, Add TOOLWINDOW
        if (mask & 4) != 0 {
            let mut style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
            style &= !WS_EX_APPWINDOW.0;
            style |= WS_EX_TOOLWINDOW.0;
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style as isize);
            // Trigger a frame redraw to apply changes
            SetWindowPos(hwnd, HWND(0), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED);
        }

        // To show in Taskbar: Remove TOOLWINDOW, Add APPWINDOW
        if (mask & 8) != 0 {
            let mut style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
            style &= !WS_EX_TOOLWINDOW.0;
            style |= WS_EX_APPWINDOW.0;
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style as isize);
            SetWindowPos(hwnd, HWND(0), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED);
        }
    }
    
    BOOL(1)
}