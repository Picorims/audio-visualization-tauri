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

use std::fs;

// the payload type must implement `Serialize` and `Clone`.
#[derive(Clone, serde::Serialize)]
struct AudioCachedPayload {
    error: bool,
    message: String
}

/**
 * Copy the audio file specified in `path` into raw PCM data.
 */
#[tauri::command]
fn cache_audio(app_handle: tauri::AppHandle, window: tauri::Window, path: &str) -> Result<(), String> {
    use tauri::api::process::{Command, CommandEvent};

    let tmp_dir = app_handle
        .path_resolver()
        .app_cache_dir()
        .expect("couldn't find cache directory");

    let out_path = tmp_dir.join("audio.raw");

    if out_path.exists() {
        fs::remove_file(out_path.to_str().expect("couldn't convert output path")).expect("could not delete existing file");
    }

    let (mut rx, mut child) = Command::new_sidecar("ffmpeg")
        .expect("failed to create `ffmpeg` binary command")
        .args([
            "-i",
            path,
            "-ar",
            "44100", //44100 Hz
            "-ac",
            "1", // mono
            "-f",
            "s16le", //PCM signed 16-bit little-endian
            "-acodec",
            "pcm_s16le",
            out_path
                .to_str()
                .expect("couldn't create ffmpeg output path"),
        ])
        .spawn()
        .expect("Failed to spawn ffmpeg");
        
    tauri::async_runtime::spawn(async move {
        // read events
        while let Some(event) = rx.recv().await {
            if let CommandEvent::Terminated(payload) = event {
                if payload.code != Some(0) {
                    window
                        .emit("audio_cached", AudioCachedPayload {error: true, message: "FFmpeg terminated with an error.".into()})
                        .expect("failed to emit audio_cached event (1)");
                } else {
                    window
                        .emit("audio_cached", AudioCachedPayload {error: false, message: "success".into()})
                        .expect("failed to emit audio_cached event (2)");
                }
            } else if let CommandEvent::Error(e) = event {
                window
                    .emit("audio_cached", AudioCachedPayload {error: false, message: e})
                    .expect("failed to emit audio_cached event (3)");
            }
        }
    });

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![cache_audio])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
