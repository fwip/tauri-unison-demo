// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::api::process::{Command,CommandEvent};
use tokio::time::{sleep, Duration};

// The time in-between requests when checking to see if the TV server is up
const SERVER_POLL_INTERVAL : Duration = Duration::from_millis(100);
// The max number of requests to make.
// TODO: Have some logic for dealing with timeout.
const SERVER_POLL_REQUESTS: usize = 100;
// An additional duration to wait before dismissing splash screen, to allow the TV app time to
// initialize and populate the UI.
const ADDITIONAL_SPLASH_DURATION: Duration = Duration::from_millis(1000);


fn main() {
    use tauri::Manager;

    tauri::Builder::default()
        .plugin(tauri_plugin_websocket::init())
        .setup(|app| {
            let name = "main.uc";

            let main_window = app.get_window("main").unwrap();
            let splashscreen_window = app.get_window("splashscreen").unwrap();
            let url = "http://localhost:8080".to_owned();

            tauri::async_runtime::spawn(async move {
                let ucm = Command::new_sidecar("ucm")
                    .expect("Failed to create `ucm` binary command (possible bundling issue?)")
                    .args(&["run.compiled", &("resources/".to_owned() + name)]);
                let (mut rx, _child) = ucm.spawn().expect("Failed to spawn command");
                tauri::async_runtime::spawn(async move {
                    while let Some(event) = rx.recv().await {
                        match event {
                            CommandEvent::Stderr(s) => println!("Stderr: {}", s),
                            CommandEvent::Stdout(s) => println!("StdOut: {}", s),
                            CommandEvent::Error(e) => println!("Error: {}", e),
                            CommandEvent::Terminated(t) => println!("Terminated: {:?}", t),
                            ce => println!("Some other command event: {:?}", ce),
                        };
                    }
                });
                let mut i = 0;
                while i < SERVER_POLL_REQUESTS { 
                    let response = reqwest::get(url.clone()).await;
                    match response {
                        Ok(_response) => {
                            break
                        },
                        Err(_e) => (), //println!("Err: {}", e),
                    }
                    i+=1;
                    sleep(SERVER_POLL_INTERVAL).await;
                }
                main_window.eval(&format!("window.location.href = '{url}';").to_owned()).expect("could not eval JS");
                sleep(ADDITIONAL_SPLASH_DURATION).await;
                splashscreen_window.close().unwrap();
                main_window.show().unwrap();
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
