use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

mod calc;

use tauri::async_runtime::JoinHandle;
use tauri::{
    AppHandle, Emitter, Manager, RunEvent, State, WindowEvent,
};
use tauri_plugin_notification::NotificationExt;

#[cfg(desktop)]
use tauri::menu::{Menu, MenuItem};
#[cfg(desktop)]
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

#[derive(Default)]
struct TimerState {
    handles: Mutex<HashMap<String, JoinHandle<()>>>,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn cancel_inner(state: &State<TimerState>, id: &str) {
    if let Some(handle) = state.handles.lock().unwrap().remove(id) {
        handle.abort();
    }
}

#[tauri::command]
async fn schedule_timer(
    id: String,
    label: String,
    fire_at_ms: u64,
    state: State<'_, TimerState>,
    app: AppHandle,
) -> Result<(), String> {
    cancel_inner(&state, &id);

    let now = now_ms();
    let delay_ms = fire_at_ms.saturating_sub(now);

    let id_for_task = id.clone();
    let label_for_task = label.clone();
    let app_for_task = app.clone();

    let handle = tauri::async_runtime::spawn(async move {
        if delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }

        let payload = serde_json::json!({
            "id": id_for_task,
            "label": label_for_task,
        });
        let _ = app_for_task.emit("timer-finished", payload);

        let _ = app_for_task
            .notification()
            .builder()
            .title("Timer finished")
            .body(&label_for_task)
            .show();

        if let Some(state) = app_for_task.try_state::<TimerState>() {
            state.handles.lock().unwrap().remove(&id_for_task);
        }
    });

    state.handles.lock().unwrap().insert(id, handle);
    Ok(())
}

#[tauri::command]
fn cancel_timer(id: String, state: State<'_, TimerState>) -> Result<(), String> {
    cancel_inner(&state, &id);
    Ok(())
}

#[tauri::command]
fn list_scheduled(state: State<TimerState>) -> Result<Vec<String>, String> {
    Ok(state.handles.lock().unwrap().keys().cloned().collect())
}

#[tauri::command]
fn app_info() -> String {
    format!("CalcTimers v{}", env!("CARGO_PKG_VERSION"))
}

#[cfg(desktop)]
fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &quit])?;

    let _tray = TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().cloned().unwrap_or_else(|| {
            tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png"))
                .expect("icon")
        }))
        .tooltip("CalcTimers")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.unminimize();
                    let _ = w.set_focus();
                }
            }
            "hide" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.hide();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(w) = app.get_webview_window("main") {
                    if w.is_visible().unwrap_or(false) {
                        let _ = w.hide();
                    } else {
                        let _ = w.show();
                        let _ = w.unminimize();
                        let _ = w.set_focus();
                    }
                }
            }
        })
        .build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }));
    }
    let builder = builder
        .manage(TimerState::default())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
            }

            #[cfg(desktop)]
            {
                build_tray(app.handle())?;
                if let Some(win) = app.get_webview_window("main") {
                    let win_clone = win.clone();
                    win.on_window_event(move |event| {
                        if let WindowEvent::CloseRequested { api, .. } = event {
                            let _ = win_clone.hide();
                            api.prevent_close();
                        }
                    });
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_info,
            schedule_timer,
            cancel_timer,
            list_scheduled,
            calc::eval_expression,
        ]);

    builder
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let RunEvent::ExitRequested { .. } = event {
                if let Some(state) = app.try_state::<TimerState>() {
                    let mut handles = state.handles.lock().unwrap();
                    for (_, h) in handles.drain() {
                        h.abort();
                    }
                }
            }
        });
}
