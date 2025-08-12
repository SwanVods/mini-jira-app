use std::sync::Mutex;
use tauri::State;

mod jira_types;
mod jira_api;
use jira_api::JiraClient;
use jira_types::{JiraIssue, WorklogResponse};

// Global state to store JIRA client
type JiraState = Mutex<Option<JiraClient>>;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn connect_to_jira(
    base_url: String,
    access_token: String,
    state: State<'_, JiraState>,
) -> Result<bool, String> {
    let client = JiraClient::new(base_url, access_token);
    
    // Test the connection
    match client.test_connection().await {
        Ok(is_connected) => {
            if is_connected {
                // Store the client in global state
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
    // Clone the client from the guard, then drop the guard
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

#[tauri::command]
async fn create_worklog(
    issue_key: String,
    description: String,
    started: String,
    time_spent: String,
    state: State<'_, JiraState>,
) -> Result<WorklogResponse, String> {
    // Clone the client from the guard, then drop the guard
    let client = {
        let jira_state = state.lock().map_err(|e| e.to_string())?;
        jira_state.as_ref().cloned()
    };
    
    match client {
        Some(client) => {
            // Parse time spent to seconds
            let time_spent_seconds = JiraClient::parse_time_to_seconds(&time_spent)
                .map_err(|e| format!("Invalid time format: {}", e))?;
            
            client.create_worklog(
                &issue_key,
                &description,
                &started,
                time_spent_seconds,
                None, // No visibility restrictions for now
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(JiraState::default())
        .invoke_handler(tauri::generate_handler![
            greet,
            connect_to_jira,
            get_assigned_issues,
            create_worklog,
            disconnect_from_jira
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
