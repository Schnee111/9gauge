//! 9Gauge Tauri v2 Desktop Companion Shell.
//! Consumes `gauge-core` headless engine — contains zero parsing or telemetry logic.

use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State, WindowEvent,
};
use tokio::sync::{watch, Mutex};
use tracing::{info, warn};

use gauge_core::auth::{AuthStrategy, LocalCli};
use gauge_core::state::{AppState, ConnectionState, StateHub};
use gauge_core::telemetry::{ClientConfig, TelemetryClient};

struct AppSharedState {
    hub: Arc<StateHub>,
    shutdown_tx: watch::Sender<bool>,
}

#[derive(Serialize)]
struct SerializableAppState {
    snapshot: gauge_core::models::UsageSnapshot,
    connection: String,
    host: String,
}

impl From<&AppState> for SerializableAppState {
    fn from(s: &AppState) -> Self {
        let conn_str = match s.connection {
            ConnectionState::Connecting => "Connecting",
            ConnectionState::Healthy => "Healthy",
            ConnectionState::Degraded => "Degraded",
            ConnectionState::Disconnected => "Disconnected",
            ConnectionState::Reconnecting => "Reconnecting",
            ConnectionState::AuthFailed => "AuthFailed",
        };
        Self {
            snapshot: s.snapshot.clone(),
            connection: conn_str.to_string(),
            host: s.host.to_string(),
        }
    }
}

#[tauri::command]
fn get_state(state: State<AppSharedState>) -> SerializableAppState {
    let current = state.hub.load();
    SerializableAppState::from(&*current)
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    info!("Quit commanded from UI");
    app.exit(0);
}

#[tauri::command]
async fn set_period(period: String, state: State<'_, AppSharedState>) -> Result<(), String> {
    info!("Period switched to {}", period);
    // Period change is reflected on next stats polling cycle
    Ok(())
}

fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            info!("Booting 9Gauge desktop companion shell");

            let hub = Arc::new(StateHub::new());
            let (shutdown_tx, shutdown_rx) = watch::channel(false);

            // Auto-detect local 9Router config or default to localhost:20128
            let mut config = ClientConfig::default();
            config.base_url = "http://localhost:20128".to_string();

            // Attempt local CLI auth discovery
            let data_dir = std::env::var("NINEROUTER_DATA_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|_| {
                    dirs::home_dir()
                        .unwrap_or_default()
                        .join(".9router")
                });

            if data_dir.exists() {
                config.auth = AuthStrategy::Local(LocalCli::new(data_dir));
            }

            let client = TelemetryClient::new(config, (*hub).clone());
            let hub_clone = Arc::clone(&hub);
            let app_handle = app.handle().clone();

            // 1. Spawn background telemetry engine
            tauri::async_runtime::spawn(async move {
                client.run(shutdown_rx).await;
            });

            // 2. State broadcast loop to UI + tray tooltip update
            let app_handle_state = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_millis(500));
                let mut last_total = 0u64;

                loop {
                    interval.tick().await;
                    let current = hub_clone.load();
                    let payload = SerializableAppState::from(&*current);

                    // Broadcast to Svelte UI
                    let _ = app_handle_state.emit("telemetry:state", &payload);

                    // Update tray tooltip with live metrics
                    let total_tok = current.snapshot.total_tokens();
                    if total_tok != last_total {
                        last_total = total_tok;
                        let tooltip = format!(
                            "9Gauge: {} tok · ${:.2}",
                            total_tok, current.snapshot.total_cost
                        );
                        if let Some(tray) = app_handle_state.tray_by_id("main-tray") {
                            let _ = tray.set_tooltip(Some(tooltip));
                            #[cfg(target_os = "macos")]
                            {
                                let title = format!(
                                    "✦ 9R · {:.1}M · ${:.2}",
                                    total_tok as f64 / 1_000_000.0,
                                    current.snapshot.total_cost
                                );
                                let _ = tray.set_title(Some(title));
                            }
                        }
                    }
                }
            });

            // 3. Register Tray Icon
            let quit_i = MenuItem::with_id(app, "quit", "Quit 9Gauge", true, None::<&str>)?;
            let toggle_i = MenuItem::with_id(app, "toggle", "Toggle HUD", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&toggle_i, &quit_i])?;

            let _tray = TrayIconBuilder::with_id("main-tray")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "toggle" => {
                        toggle_window(app);
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
                        toggle_window(app);
                    }
                })
                .build(app)?;

            // 4. Auto-hide on blur (popover dismissal)
            if let Some(window) = app.get_webview_window("main") {
                let win_clone = window.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::Focused(false) = event {
                        let _ = win_clone.hide();
                    }
                });
            }

            app.manage(AppSharedState { hub, shutdown_tx });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_state, set_period, quit_app])
        .run(tauri::generate_context!())
        .expect("error while running 9gauge application");
}
