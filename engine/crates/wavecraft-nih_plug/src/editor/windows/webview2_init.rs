//! Windows WebView2 initialization wiring.
//!
//! Owns environment/controller initialization, settings setup,
//! content loading orchestration, and readiness message pumping.

use std::sync::{Arc, Mutex};

use nih_plug::prelude::*;
use webview2_com::Microsoft::Web::WebView2::Win32::{ICoreWebView2, ICoreWebView2Controller};
use webview2_com::{
    CreateCoreWebView2ControllerCompletedHandler, CreateCoreWebView2EnvironmentCompletedHandler,
};
use windows::Win32::Foundation::{E_FAIL, HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage,
};
use windows::core::{HSTRING, PCWSTR};

use super::JsonIpcHandler;
use super::content;
use super::ipc_bridge;

/// Initialize WebView2 using the COM API.
pub(super) fn initialize_webview2(
    hwnd: HWND,
    controller: Arc<Mutex<Option<ICoreWebView2Controller>>>,
    webview: Arc<Mutex<Option<ICoreWebView2>>>,
    handler: Arc<Mutex<dyn JsonIpcHandler>>,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let user_data_folder = std::env::temp_dir().join("Wavecraft_WebView2_Data");
    let user_data_path = HSTRING::from(user_data_folder.to_string_lossy().as_ref());

    let controller_clone = controller.clone();
    let webview_clone = webview.clone();
    let handler_clone = handler;

    let env_handler = CreateCoreWebView2EnvironmentCompletedHandler::create(Box::new(
        move |result, environment| {
            if result.is_err() {
                nih_error!("Failed to create WebView2 environment: {:?}", result);
                return E_FAIL;
            }

            let environment = match environment {
                Some(env) => env,
                None => {
                    nih_error!("WebView2 environment is null");
                    return E_FAIL;
                }
            };

            let controller_inner = controller_clone.clone();
            let webview_inner = webview_clone.clone();
            let handler_inner = handler_clone.clone();

            let controller_handler = CreateCoreWebView2ControllerCompletedHandler::create(
                Box::new(move |result, ctrl| {
                    if result.is_err() {
                        nih_error!("Failed to create WebView2 controller: {:?}", result);
                        return E_FAIL;
                    }

                    let ctrl = match ctrl {
                        Some(c) => c,
                        None => {
                            nih_error!("WebView2 controller is null");
                            return E_FAIL;
                        }
                    };

                    let bounds = RECT {
                        left: 0,
                        top: 0,
                        right: width as i32,
                        bottom: height as i32,
                    };
                    unsafe {
                        let _ = ctrl.SetBounds(bounds);
                        let _ = ctrl.SetIsVisible(true);
                    }

                    let wv = unsafe {
                        match ctrl.CoreWebView2() {
                            Ok(w) => w,
                            Err(e) => {
                                nih_error!("Failed to get CoreWebView2: {:?}", e);
                                return E_FAIL;
                            }
                        }
                    };

                    configure_webview_settings(&wv);
                    ipc_bridge::setup_web_message_handler(&wv, handler_inner.clone());
                    content::inject_ipc_script(&wv);
                    content::load_ui_content(&wv);

                    {
                        let mut ctrl_lock = controller_inner.lock().unwrap();
                        *ctrl_lock = Some(ctrl.clone());
                    }
                    {
                        let mut wv_lock = webview_inner.lock().unwrap();
                        *wv_lock = Some(wv);
                    }

                    nih_log!("WebView2 initialized successfully");
                    windows::core::HRESULT(0)
                }),
            );

            unsafe {
                let _ = environment.CreateCoreWebView2Controller(hwnd, &controller_handler);
            }

            windows::core::HRESULT(0)
        },
    ));

    unsafe {
        webview2_com::CreateCoreWebView2EnvironmentWithOptions(
            PCWSTR::null(),
            PCWSTR::from_raw(user_data_path.as_ptr()),
            None,
            &env_handler,
        )
        .map_err(|e| format!("CreateCoreWebView2Environment failed: {:?}", e))?;
    }

    pump_messages_until_ready(&webview, std::time::Duration::from_secs(5));

    Ok(())
}

/// Configure WebView2 settings.
fn configure_webview_settings(webview: &ICoreWebView2) {
    unsafe {
        if let Ok(settings) = webview.Settings() {
            #[cfg(debug_assertions)]
            let _ = settings.SetAreDevToolsEnabled(true);
            #[cfg(not(debug_assertions))]
            let _ = settings.SetAreDevToolsEnabled(false);
            #[cfg(not(debug_assertions))]
            let _ = settings.SetAreDefaultContextMenusEnabled(false);
            let _ = settings.SetIsScriptEnabled(true);
            let _ = settings.SetIsWebMessageEnabled(true);
        }
    }
}

/// Pump Windows messages until WebView2 is ready or timeout.
fn pump_messages_until_ready(
    webview: &Arc<Mutex<Option<ICoreWebView2>>>,
    timeout: std::time::Duration,
) {
    let start = std::time::Instant::now();

    loop {
        {
            let lock = webview.lock().unwrap();
            if lock.is_some() {
                break;
            }
        }

        if start.elapsed() > timeout {
            nih_error!("WebView2 initialization timed out");
            break;
        }

        unsafe {
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, HWND::default(), 0, 0, PM_REMOVE).as_bool() {
                if msg.message == windows::Win32::UI::WindowsAndMessaging::WM_QUIT {
                    break;
                }
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
