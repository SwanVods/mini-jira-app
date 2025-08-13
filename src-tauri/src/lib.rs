use std::sync::Mutex;
use tauri::{AppHandle, Manager, State, Wry, Emitter};
use tauri::WindowEvent;
use tauri::tray::TrayIconEvent;
use tauri::menu::{Menu, MenuItem};
use tokio::time::{interval, Duration};
use chrono::{Local, Timelike};

mod jira_types;
mod jira_api;
use jira_api::JiraClient;
use jira_types::{JiraIssue, WorklogResponse};

type JiraState = Mutex<Option<JiraClient>>;

async fn start_notification_scheduler(app_handle: AppHandle<Wry>) {
    let mut interval = interval(Duration::from_secs(60));
    
    loop {
        interval.tick().await;
        
        let now = Local::now();
        if now.hour() == 17 && now.minute() == 0 {
            if let Some(main_window) = app_handle.get_webview_window("main") {
                if let Err(e) = main_window.emit("daily-reminder", ()) {
                    eprintln!("Failed to emit daily reminder event: {}", e);
                }
            }
        }
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command(rename_all = "camelCase")]
async fn connect_to_jira(
    base_url: String,
    access_token: String,
    state: State<'_, JiraState>,
) -> Result<bool, String> {
    let client = JiraClient::new(base_url, access_token);
    
    match client.test_connection().await {
        Ok(is_connected) => {
            if is_connected {
                let mut jira_state = state.lock().map_err(|e| e.to_string())?;
                *jira_state = Some(client);
                Ok(true)
            } else {
                Err("Failed to connect to JIRA".to_string())
            }
        }
        Err(e) => Err(format!("Connection error: {}", e)),
    }
}

#[tauri::command]
async fn get_assigned_issues(
    state: State<'_, JiraState>,
) -> Result<Vec<JiraIssue>, String> {
    let client = {
        let jira_state = state.lock().map_err(|e| e.to_string())?;
        jira_state.as_ref().cloned()
    };
    
    match client {
        Some(client) => {
            client.get_assigned_issues()
                .await
                .map_err(|e| format!("Failed to get issues: {}", e))
        }
        None => Err("Not connected to JIRA".to_string()),
    }
}

#[tauri::command(rename_all = "camelCase")]
async fn create_worklog(
    issue_key: String,
    description: String,
    started: String,
    time_spent: String,
    state: State<'_, JiraState>,
) -> Result<WorklogResponse, String> {
    let client = {
        let jira_state = state.lock().map_err(|e| e.to_string())?;
        jira_state.as_ref().cloned()
    };
    
    match client {
        Some(client) => {
            let time_spent_seconds = JiraClient::parse_time_to_seconds(&time_spent)
                .map_err(|e| format!("Invalid time format: {}", e))?;
            
            client.create_worklog(
                &issue_key,
                &description,
                &started,
                time_spent_seconds,
                None,
            )
            .await
            .map_err(|e| format!("Failed to create worklog: {}", e))
        }
        None => Err("Not connected to JIRA".to_string()),
    }
}

#[tauri::command]
async fn disconnect_from_jira(
    state: State<'_, JiraState>,
) -> Result<(), String> {
    let mut jira_state = state.lock().map_err(|e| e.to_string())?;
    *jira_state = None;
    Ok(())
}

#[tauri::command]
async fn show_main_window(app_handle: AppHandle<Wry>) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn hide_to_tray(app_handle: AppHandle<Wry>) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn send_test_notification(_app_handle: AppHandle<Wry>) -> Result<(), String> {
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let show_item = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let hide_item = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
            let separator1 = MenuItem::with_id(app, "separator1", "---", false, None::<&str>)?;
            let test_notification_item = MenuItem::with_id(app, "test_notification", "Test Notification", true, None::<&str>)?;
            let separator2 = MenuItem::with_id(app, "separator2", "---", false, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            
            let menu = Menu::with_items(app, &[
                &show_item,
                &hide_item,
                &separator1,
                &test_notification_item,
                &separator2,
                &quit_item,
            ])?;

            app.tray_by_id("main")
                .expect("Failed to get tray")
                .set_menu(Some(menu))?;

            let app_handle = app.handle().clone();
            
            tauri::async_runtime::spawn(async move {
                start_notification_scheduler(app_handle).await;
            });
            
            Ok(())
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "hide" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            "test_notification" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.emit("test-notification", ());
                }
            }
            "quit" => {
                std::process::exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|app, event| match event {
            TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                button_state: tauri::tray::MouseButtonState::Up,
                ..
            } => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            _ => {}
        })
        .on_window_event(|_window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                _window.hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .manage(JiraState::default())
        .invoke_handler(tauri::generate_handler![
            greet,
            connect_to_jira,
            get_assigned_issues,
            create_worklog,
            disconnect_from_jira,
            show_main_window,
            hide_to_tray,
            send_test_notification
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
