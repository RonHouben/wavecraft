//! Windows runtime and HWND helper functions for WebView2 editor initialization.

use nih_plug::prelude::*;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, FillRect, HBRUSH, PAINTSTRUCT};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CS_HREDRAW, CS_VREDRAW, CreateWindowExW, DefWindowProcW, GetClientRect, HMENU,
    RegisterClassExW, WINDOW_EX_STYLE, WM_PAINT, WNDCLASSEXW, WS_CHILD, WS_VISIBLE,
};
use windows::core::PCWSTR;

/// Check if WebView2 runtime is installed.
pub fn check_webview2_runtime() -> Result<(), String> {
    match webview2_com::get_available_core_webview2_browser_version_string(PCWSTR::null()) {
        Ok(version) => {
            if version.is_empty() {
                Err("WebView2 runtime not installed. Please install it from:\n\
                     https://developer.microsoft.com/microsoft-edge/webview2/"
                    .to_string())
            } else {
                Ok(())
            }
        }
        Err(e) => Err(format!(
            "Failed to detect WebView2 runtime: {:?}\n\
             Please install it from:\n\
             https://developer.microsoft.com/microsoft-edge/webview2/",
            e
        )),
    }
}

/// Extract HWND from ParentWindowHandle.
pub unsafe fn get_parent_hwnd(handle: ParentWindowHandle) -> Result<HWND, String> {
    match handle {
        ParentWindowHandle::Win32Hwnd(hwnd_ptr) => {
            if hwnd_ptr.is_null() {
                return Err("Parent HWND is null".to_string());
            }
            Ok(HWND(hwnd_ptr as isize))
        }
        _ => Err("Expected Win32Hwnd parent on Windows".to_string()),
    }
}

/// Create a child window for hosting the WebView2 control.
pub unsafe fn create_child_window(parent: HWND, width: u32, height: u32) -> Result<HWND, String> {
    let class_name = windows::core::w!("WavecraftWebView");
    let h_instance =
        GetModuleHandleW(None).map_err(|e| format!("GetModuleHandleW failed: {}", e))?;

    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: h_instance.into(),
        hIcon: Default::default(),
        hCursor: Default::default(),
        hbrBackground: HBRUSH(6_isize),
        lpszMenuName: windows::core::PCWSTR::null(),
        lpszClassName: class_name,
        hIconSm: Default::default(),
    };

    let atom = RegisterClassExW(&wc);
    if atom == 0 {
        nih_log!("Window class registration returned 0 (may already exist)");
    }

    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        class_name,
        windows::core::w!("Wavecraft Editor"),
        WS_CHILD | WS_VISIBLE,
        0,
        0,
        width as i32,
        height as i32,
        parent,
        HMENU::default(),
        h_instance,
        None,
    )
    .map_err(|e| format!("CreateWindowExW failed: {}", e))?;

    Ok(hwnd)
}

/// Window procedure for the child window.
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            if !hdc.is_invalid() {
                let mut rect = RECT::default();
                let _ = GetClientRect(hwnd, &mut rect);
                let brush = windows::Win32::Graphics::Gdi::GetSysColorBrush(
                    windows::Win32::UI::WindowsAndMessaging::COLOR_WINDOW,
                );
                let _ = FillRect(hdc, &rect, brush);
            }
            EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
