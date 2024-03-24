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
    message: String,
}

static mut FFT_CACHE: Vec<Vec<rustfft::num_complex::Complex<f32>>> = Vec::new();
const FFT_SIZE: usize = 4096;

/**
 * Copy the audio file specified in `path` into raw PCM data.
 */
#[tauri::command]
fn cache_audio(
    app_handle: tauri::AppHandle,
    window: tauri::Window,
    path: &str,
) -> Result<(), String> {
    use tauri::api::process::{Command, CommandEvent};

    let tmp_dir = app_handle
        .path_resolver()
        .app_cache_dir()
        .expect("couldn't find cache directory");

    let out_path = tmp_dir.join("audio.raw");

    if out_path.exists() {
        fs::remove_file(out_path.to_str().expect("couldn't convert output path"))
            .expect("could not delete existing file");
    }

    let (mut rx, _child) = Command::new_sidecar("ffmpeg")
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
                        .emit(
                            "audio_cached",
                            AudioCachedPayload {
                                error: true,
                                message: "FFmpeg terminated with an error.".into(),
                            },
                        )
                        .expect("failed to emit audio_cached event (1)");
                } else {
                    match cache_fft(&out_path) {
                        Ok(()) => {
                            window
                                .emit(
                                    "audio_cached",
                                    AudioCachedPayload {
                                        error: false,
                                        message: "success".into(),
                                    },
                                )
                                .expect("failed to emit audio_cached event (2)");
                        }
                        Err(e) => {
                            window
                                .emit(
                                    "audio_cached",
                                    AudioCachedPayload {
                                        error: false,
                                        message: e,
                                    },
                                )
                                .expect("failed to emit audio_cached event (3)");
                        }
                    }
                }
            } else if let CommandEvent::Error(e) = event {
                window
                    .emit(
                        "audio_cached",
                        AudioCachedPayload {
                            error: false,
                            message: e,
                        },
                    )
                    .expect("failed to emit audio_cached event (4)");
            }
        }
    });

    Ok(())
}

/**
 * Cache FFT for each frame
 */
fn cache_fft(path: &std::path::Path) -> Result<(), String> {
    use rustfft::{num_complex::Complex, FftPlanner};
    use std::fs::File;
    use std::io::Read; // necessary for file.by_ref()

    const RATE: usize = 44100;
    let mut file = File::open(path).expect("Could not open file for FFT processing");

    // for future: https://stackoverflow.com/questions/55555538/what-is-the-correct-way-to-read-a-binary-file-in-chunks-of-a-fixed-size-and-stor

    // put file in memory
    let mut buffer = Vec::new();
    file.by_ref()
        .read_to_end(&mut buffer)
        .expect("Couldn't read PCM data");
    let len = buffer.len()/2; // /2 because two u8 => u16 = 1 value !!!
    let duration = len / RATE; // in seconds

    let mut frame: usize = 0;
    const FPS: usize = 60;

    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(FFT_SIZE);

    loop {
        // process 1 frame
        frame += 1;
        println!("Frame: {}", frame);
        if frame / FPS > duration {
            break;
        }
        let center: isize = (RATE / FPS * frame) as isize;
        let from: isize = center - (FFT_SIZE / 2) as isize;
        let to: isize = center + (FFT_SIZE / 2) as isize;
        let mut buffer_fft: Vec<Complex<f32>> = Vec::new();

        println!("From: {}, To: {}", from, to);
        for i in from..to
        /*excluded*/
        {
            if i < 0 || i > (buffer.len()/2 - 1) as isize {
                buffer_fft.push(Complex { re: 0.0, im: 0.0 });
            } else {
                let j: usize = i.try_into().unwrap();
                buffer_fft.push(Complex {
                    re: ((buffer[2*j] as u16) << 8 | buffer[2*j+1] as u16).try_into().unwrap(),
                    im: 0.0,
                });
            }
        }

        fft.process(&mut buffer_fft);
        unsafe {
            FFT_CACHE.push(buffer_fft.clone());
        }
    }

    Ok(())
}

#[tauri::command]
fn get_frame_fft(frame: usize) -> Vec<f32> {
    let mut vector: Vec::<f32> = Vec::new();
    let cache_clone: Vec<rustfft::num_complex::Complex<f32>>; 
    unsafe {
        cache_clone = FFT_CACHE[frame].clone();
    }

    // println!("{cache_clone}");

    for i in 0..cache_clone.len() {
        vector.push((cache_clone[i]*1.0/(FFT_SIZE as f32).sqrt()).norm());
    }

    return vector;
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![cache_audio, get_frame_fft])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
