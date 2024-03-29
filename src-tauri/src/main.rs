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

use spectrum_analyzer::{Frequency, FrequencySpectrum, FrequencyValue};

// the payload type must implement `Serialize` and `Clone`.
#[derive(Clone, serde::Serialize)]
struct AudioCachedPayload {
    error: bool,
    message: String,
}

static mut FFT_CACHE: Vec<FrequencySpectrum> = Vec::new();
const FFT_SIZE: usize = 16384;

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
    use spectrum_analyzer::scaling::divide_by_N;
    use spectrum_analyzer::windows::hann_window;
    use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};
    
    use std::fs::File;
    use std::io::Read; // necessary for file.by_ref()

    const RATE: usize = 44100;
    let mut file = File::open(path).expect("Could not open file for FFT processing");

    // for future: https://stackoverflow.com/questions/55555538/what-is-the-correct-way-to-read-a-binary-file-in-chunks-of-a-fixed-size-and-stor

    // put file in memory
    let mut buffer_read = Vec::new();
    file.by_ref()
        .read_to_end(&mut buffer_read)
        .expect("Couldn't read PCM data");
    
    // convert to i16 vector (two u8 are already a i16 value as per ffmpeg arguments)
    let mut buffer: Vec<i16> = Vec::new();
    for i in 0..buffer_read.len() / 2 {
        let mut bytes: [u8; 2] = [0; 2];
        bytes.copy_from_slice(&buffer_read[i * 2..i * 2 + 2]);
        let value = i16::from_le_bytes(bytes);
        buffer.push(value);
    }

    let len = buffer.len();
    let duration = len / RATE; // in seconds

    let mut frame: usize = 0;
    const FPS: usize = 60;

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
        let mut buffer_fft: Vec<f32> = Vec::new();

        println!("From: {}, To: {}", from, to);
        for i in from..to
        /*excluded*/
        {
            if i < 0 || i > (buffer.len() - 1) as isize {
                buffer_fft.push(0.0);
            } else {
                let j: usize = i.try_into().unwrap();
                buffer_fft.push(buffer[j].try_into().unwrap());
            }
        }

        let hann_window = hann_window(&buffer_fft);
        let spectrum = samples_fft_to_spectrum(
            &hann_window,
            RATE as u32,
            FrequencyLimit::All,
            Some(&divide_by_N)
        ).unwrap();
        unsafe {
            FFT_CACHE.push(spectrum);
        }
    }

    Ok(())
}

#[tauri::command]
fn get_frame_fft(frame: usize) -> Vec<(f32, f32)> {
    let &cached_spectrum; 
    let length ;
    unsafe {
        length = FFT_CACHE.len();
    }
    if frame >= length {
        return vec![(-1.0, -1.0);FFT_SIZE];
    }
    unsafe {
        cached_spectrum = &FFT_CACHE[frame];
    }

    return cached_spectrum
        .data()
        .iter()
        .map(|f| (f.0.val(), f.1.val()))
        .collect();
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![cache_audio, get_frame_fft])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
