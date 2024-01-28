// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::api::process::{Command,CommandEvent};
use tokio::time::{sleep, Duration};


//#[tauri::command]
//async fn launch(name: &str) -> Result<String, ()> {
//    let url = "http://localhost:8080".to_owned();
//    let ucm = Command::new_sidecar("ucm")
//        .expect("Failed to create `ucm` binary command (possible bundling issue?)")
//        .args(&["run.compiled", &("resources/".to_owned() + name)]);
//
//    let (mut rx, mut child) = ucm.spawn().expect("Failed to spawn command");
//    println!("Spawned {}", name);
//    tauri::async_runtime::spawn(async move {
//        while let Some(event) = rx.recv().await {
//            match event {
//                CommandEvent::Stderr(s) => println!("Stderr: {}", s),
//                CommandEvent::Stdout(s) => println!("StdOut: {}", s),
//                CommandEvent::Error(e) => println!("Error: {}", e),
//                CommandEvent::Terminated(_) => println!("Terminated"),
//                ce => println!("Some other command event: {:?}", ce),
//            };
//        }
//    });
//    let mut i = 0;
//    while i < 10 { 
//        let response = reqwest::get(url.clone()).await;
//        match response {
//            Ok(response) => {
//                println!("Ok! {:?}", response);
//                break
//            },
//            Err(e) => (), //println!("Err: {}", e),
//        }
//        i+=1;
//        sleep(Duration::from_millis(100)).await;
//
//    }
//
//    Ok(url)
//}

fn main() {
    use tauri::Manager;


    tauri::Builder::default()
        .plugin(tauri_plugin_websocket::init())
        //.invoke_handler(tauri::generate_handler![launch])
        .setup(|app| {
            let name = "main";

            let main_window = app.get_window("main").unwrap();
            let splashscreen_window = app.get_window("splashscreen").unwrap();

            tauri::async_runtime::spawn(async move {
                let url = "http://localhost:8080".to_owned();
                let ucm = Command::new_sidecar("ucm")
                    .expect("Failed to create `ucm` binary command (possible bundling issue?)")
                    .args(&["run.compiled", &("resources/".to_owned() + name)]);
                let (mut rx, mut child) = ucm.spawn().expect("Failed to spawn command");
                while let Some(event) = rx.recv().await {
                    match event {
                        CommandEvent::Stderr(s) => println!("Stderr: {}", s),
                        CommandEvent::Stdout(s) => println!("StdOut: {}", s),
                        CommandEvent::Error(e) => println!("Error: {}", e),
                        CommandEvent::Terminated(_) => println!("Terminated"),
                        ce => println!("Some other command event: {:?}", ce),
                    };
                }
                let mut i = 0;
                while i < 10 { 
                    let response = reqwest::get(url.clone()).await;
                    match response {
                        Ok(response) => {
                            println!("Ok! {:?}", response);
                            break
                        },
                        Err(e) => (), //println!("Err: {}", e),
                    }
                    i+=1;
                    sleep(Duration::from_millis(100)).await;
                }
                main_window.eval(&format!("location.href = {url};").to_owned()).expect("could not eval JS");
                sleep(Duration::from_millis(100)).await;
                splashscreen_window.close().unwrap();
                main_window.show().unwrap();
            });


            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    //child.kill().expect("Could not kill ucm child process");

}
