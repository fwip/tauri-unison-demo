// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use tauri::Manager;
use tauri::api::process::{Command,CommandEvent};
use tokio::time::{sleep, Duration};

// The time in-between requests when checking to see if the TV server is up
const SERVER_POLL_INTERVAL : Duration = Duration::from_millis(100);
// Max duration to wait before giving up
const SERVER_POLL_TIMEOUT: Duration = Duration::from_secs(60);
// An additional duration to wait before dismissing splash screen, to allow the TV app time to
// initialize and populate the UI.
const ADDITIONAL_SPLASH_DURATION: Duration = Duration::from_millis(1000);

async fn wait_until_server_is_up(url: String) -> reqwest::Result<reqwest::Response>{
    // Wait until server is running
    let start = std::time::Instant::now();
    loop { 
        let response = reqwest::get(url.clone()).await;
        match response {
            response@Ok(_) => {
                return response;
            },
            e@Err(_) => {
                let now = std::time::Instant::now();
                if (now - start) >= SERVER_POLL_TIMEOUT {
                    return e;
                }
            }
        }
        sleep(SERVER_POLL_INTERVAL).await;
    }
}

async fn simple_ucm_monitor(rx: &mut tauri::async_runtime::Receiver<CommandEvent>) -> () {
    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stderr(s) => println!("UCM Stderr: {}", s.trim_end()),
            CommandEvent::Stdout(s) => println!("UCM StdOut: {}", s.trim_end()),
            CommandEvent::Error(e) => println!("UCM Error: {}", e.trim_end()),
            CommandEvent::Terminated(t) => {
                println!("UCM Terminated: {:?}", t);
                return ()
            },
            ce => println!("Some other command event: {:?}", ce),
        };
    }
}

fn setup_ucm(filename: String, app: &mut tauri::App) -> Result<(), Box<dyn Error>> {

    let url = "http://localhost:8080".to_owned();

    let main_window = app.get_window("main").unwrap();
    let splashscreen_window = app.get_window("splashscreen").unwrap();

    tauri::async_runtime::spawn(async move {
        // Run Unison server
        let ucm = Command::new_sidecar("ucm")
            .expect("Failed to create `ucm` binary command (possible bundling issue?)")
            .args(&["run.compiled", &("resources/".to_owned() + &filename)]);
        let (mut rx, _child) = ucm.spawn().expect("Failed to spawn command");

        // Simple monitoring of the UCM process
        tauri::async_runtime::spawn(async move {
            simple_ucm_monitor(&mut rx).await;
        });

        // Wait until server is running
        match wait_until_server_is_up(url.clone()).await {
            Ok(_response) => {
                // Redirect to the local server, and throw away the splash screen.
                main_window.eval(&format!("window.location.href = '{url}';").to_owned()).expect("could not eval JS");
                sleep(ADDITIONAL_SPLASH_DURATION).await;
                splashscreen_window.close().unwrap();
                main_window.show().unwrap();
            },
            Err(e) => {
                println!("Error: {e}");
                //child.kill().expect("destroying ucm process");
                main_window.eval(&format!("window.location.href = 'error.html';").to_owned()).expect("could not eval JS");
                sleep(ADDITIONAL_SPLASH_DURATION).await;
                splashscreen_window.close().unwrap();
                main_window.eval(&format!("error('{e}');")).expect("eval error JS");
                main_window.show().unwrap();
            }
        }
    });
    Ok(())
}

fn main() {
    let filename = "main.uc".to_owned();
    tauri::Builder::default()
        .plugin(tauri_plugin_websocket::init())
        .setup(|app| { setup_ucm(filename, app) } )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
