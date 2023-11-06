// audio-visualization-tauri is an audio visualization experiment
// Copyright (C) 2023  Charly Schmidt alias Picorims<picorims.contact@gmail.com>

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn cache_audio(app_handle: tauri::AppHandle, path: &str) -> Result<(), String> {
    use tauri::api::process::{Command, CommandEvent};

    let tmp_dir = app_handle
        .path_resolver()
        .app_cache_dir()
        .expect("couldn't find cache directory");

    let (mut rx, mut child) = Command::new_sidecar("ffmpeg")
        .expect("failed to create `ffmpeg` binary command")
        .args([
            "-i",
            path,
            "-f",
            "s16le",
            "-acodec",
            "pcm_s16le",
            tmp_dir
                .join("audio.raw")
                .to_str()
                .expect("couldn't create ffmpeg output path"),
        ])
        .spawn()
        .expect("Failed to spawn ffmpeg");

    // tauri::async_runtime::spawn(async move {
    //     // read events such as stdout
    //     while let Some(event) = rx.recv().await {
    //         if let CommandEvent::Stdout(line) = event {
    //             window
    //                 .emit("message", Some(format!("'{}'", line)))
    //                 .expect("failed to emit event");
    //         }
    //     }
    // });

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .invoke_handler(tauri::generate_handler![cache_audio])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
