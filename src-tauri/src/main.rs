// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused)]

use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, Stream};
use std::fs::Metadata;
// use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
// use tauri::api::process::Command as CMD;
use chrono::{self, DateTime, Utc};

use rustfft::{num_complex::Complex, FftPlanner};
use std::sync::Mutex;
use std::{fs::metadata, fs::File, io::BufWriter, process::Command};
use tauri::command::CommandItem;
use tauri::State;
use tauri::{Manager, Window};

mod database;

struct WavMetadata {
    created: &'static str,
    sample_rate: i32,
}

// the payload type must implement `Serialize` and `Clone`.
#[derive(Clone, serde::Serialize)]
struct Payload {
    message: String,
}

const INPUT_WAV_PATH: &str = "assets/input.wav";
const ASSETS_PATH: &str = "assets/";
// const OUTPUT_WAV_PATH: &str = "assets/output.wav";

#[derive(Debug)]
struct Mbool(Mutex<bool>);

struct AlwaysOnTop(Mutex<bool>);

#[derive(Debug)]
struct IPCstring(Mutex<String>);

struct Mwriter(Mutex<WavWriterHandle>);

struct NStream(Stream);
unsafe impl std::marker::Send for NStream {}
struct Mstream(Mutex<NStream>);

fn main() {
    tauri::Builder::default()
        .manage(Mbool(Default::default()))
        .manage(AlwaysOnTop(Default::default()))
        .manage(Mstream({
            let opt = Opt::parse();
            let host = cpal::default_host();

            // Set up the input device and stream with the default input config.
            let device = if opt.device == "default" {
                host.default_input_device()
            } else {
                host.input_devices()
                    .expect("")
                    .find(|x| x.name().map(|y| y == opt.device).unwrap_or(false))
            }
            .expect("failed to find input device");

            let config = device
                .default_input_config()
                .expect("Failed to get default input config");

            let path: &str = INPUT_WAV_PATH.clone();

            let spec = wav_spec_from_config(&config);
            let writer0 = hound::WavWriter::create(path, spec).expect("writer failed");
            let writer = Arc::new(Mutex::new(Some(writer0)));

            // // Run the input stream on a separate thread.
            let writer_2 = writer.clone();

            let err_fn = move |err| {
                eprintln!("an error occurred on stream: {}", err);
            };

            let stream = device
                .build_input_stream(
                    &config.into(),
                    move |data, _: &_| write_input_data::<f32, f32>(data, &writer_2),
                    err_fn,
                    None,
                )
                .expect("stream fail");

            Mutex::new(NStream(stream))
        }))
        .manage(Mwriter({
            let opt = Opt::parse();
            let host = cpal::default_host();

            // Set up the input device and stream with the default input config.
            let device = if opt.device == "default" {
                host.default_input_device()
            } else {
                host.input_devices()
                    .expect("")
                    .find(|x| x.name().map(|y| y == opt.device).unwrap_or(false))
            }
            .expect("failed to find input device");

            let config = device
                .default_input_config()
                .expect("Failed to get default input config");

            let path: &str = INPUT_WAV_PATH.clone();

            let spec = wav_spec_from_config(&config);

            let writer0 = hound::WavWriter::create(path, spec).expect("writer failed");
            let writer = Arc::new(Mutex::new(Some(writer0)));

            Mutex::new(writer)
        }))
        .setup(|app| {
            let main_window = app.get_window("main").unwrap();
            if let Ok(items) = database::get_all(&app.app_handle()) {
                if items[2] == "true" {
                    main_window.set_always_on_top(true);
                } else {
                    main_window.set_always_on_top(false);
                }
            };
            main_window.set_always_on_top(true);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            record,
            stop_recording,
            always_on_top_true,
            always_on_top_false,
            set_ipc,
            get_ipc,
            get_prefs,
            set_accent,
            set_dark_mode,
            set_always_on_top,
            file_metadata,
            get_wav_data,
            get_stft_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Parser, Debug)]
#[command(version, about = "CPAL record_wav example", long_about = None)]
pub struct Opt {
    /// The audio device to use
    #[arg(short, long, default_value_t = String::from("default"))]
    device: String,
    // /// Use the JACK host
    // #[cfg(all(
    //     any(
    //         target_os = "linux",
    //         target_os = "dragonfly",
    //         target_os = "freebsd",
    //         target_os = "netbsd",
    //     ),
    //     feature = "jack"
    // ))]
    // #[arg(short, long)]
    // #[allow(dead_code)]
    // jack: bool,
}

fn sample_format(format: cpal::SampleFormat) -> hound::SampleFormat {
    if format.is_float() {
        hound::SampleFormat::Float
    } else {
        hound::SampleFormat::Int
    }
}

fn wav_spec_from_config(config: &cpal::SupportedStreamConfig) -> hound::WavSpec {
    hound::WavSpec {
        channels: config.channels() as _,
        sample_rate: config.sample_rate().0 as _,
        bits_per_sample: (config.sample_format().sample_size() * 8) as _,
        sample_format: sample_format(config.sample_format()),
    }
}

type WavWriterHandle = Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>;

fn write_input_data<T, U>(input: &[T], writer: &WavWriterHandle)
where
    T: Sample,
    U: Sample + hound::Sample + FromSample<T>,
{
    if let Ok(mut guard) = writer.try_lock() {
        if let Some(writer) = guard.as_mut() {
            for &sample in input.iter() {
                let sample: U = U::from_sample(sample);
                writer.write_sample(sample).ok();
            }
        }
    }
}

#[tauri::command]
fn set_ipc(name: &str, ipc: State<IPCstring>) {
    let mut newipc = ipc.0.lock().unwrap();
    *newipc = name.to_string();
}

#[tauri::command]
fn get_prefs(app_handle: tauri::AppHandle, window: Window) {
    if let Ok(items) = database::get_all(&app_handle) {
        window.emit("prefs", items).expect("failed to emit event");
    };
}

#[tauri::command]
fn set_accent(accent: &str, app_handle: tauri::AppHandle) {
    let r = database::set_accent(accent.to_string(), &app_handle);
}
#[tauri::command]
fn set_dark_mode(dark_mode: &str, app_handle: tauri::AppHandle) {
    let dark = dark_mode == "dark"; // goes to false if not
    let r = database::set_dark_mode(dark, &app_handle);
}
#[tauri::command]
fn set_always_on_top(on_top: &str, app_handle: tauri::AppHandle) {
    let top = on_top == "true";
    let r = database::set_on_top(top, &app_handle);
}

#[tauri::command]
fn get_ipc(ipc: State<IPCstring>, window: Window) {
    let newipc = ipc.0.lock().unwrap().clone();
    let s = newipc.to_string();
    window
        .emit("set_initial_ipc", s)
        .expect("failed to emit event");
}

#[tauri::command]
fn always_on_top_false(window: Window) {
    window.set_always_on_top(false);
}

#[tauri::command]
fn always_on_top_true(window: Window) {
    window.set_always_on_top(true);
}

#[tauri::command]
fn stop_recording(
    _name: &str,
    is_recording: State<Mbool>,
    mwriter: State<Mwriter>,
    mstream: State<Mstream>,
) {
    let mut nt = is_recording.0.lock().unwrap();
    *nt = false;
    let writer = mwriter.0.lock().unwrap();
    let stream = mstream.0.lock().unwrap();

    stream.0.pause().expect("failed to pause stream");

    if let Some(i) = writer.clone().lock().unwrap().take() {
        i.finalize().expect("writer failed to finalize");
    } else {
        println!("writer failed");
    }
}

#[tauri::command]
fn file_metadata(path: &str, window: Window) -> String {
    let meta = metadata(path);

    if meta.is_ok() {
        iso8601(&meta.unwrap().created().unwrap()).to_string()
    } else {
        "err".to_string()
    }
}

fn iso8601(st: &std::time::SystemTime) -> String {
    let dt: DateTime<Utc> = st.clone().into();
    format!("{}", dt.format("%+"))
    // format!("{}", dt.format("%Y-%m-%d--%H:%M:%S"))
    // formats like "2001-07-08T00:34:60.026490+09:30"
}

#[tauri::command]
fn record(
    name: &str,
    is_recording: State<Mbool>,
    mwriter: State<Mwriter>,
    mstream: State<Mstream>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let mut nt = is_recording.0.lock().unwrap();
    let mut writer = mwriter.0.lock().unwrap();
    let mut stream = mstream.0.lock().unwrap();
    *nt = true;

    let opt = Opt::parse();
    let host = cpal::default_host();

    // Set up the input device and stream with the default input config.
    let device = if opt.device == "default" {
        host.default_input_device()
    } else {
        host.input_devices()
            .expect("")
            .find(|x| x.name().map(|y| y == opt.device).unwrap_or(false))
    }
    .expect("failed to find input device");

    let config = device
        .default_input_config()
        .expect("Failed to get default input config");

    let input_path = app_handle
        .path_resolver()
        .resolve_resource(ASSETS_PATH)
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();
    // println!("{:?}", input_path.clone() + "/" + name);

    let spec = wav_spec_from_config(&config);
    let writer0 = hound::WavWriter::create(input_path + "/" + name, spec).expect("writer failed");
    *writer = Arc::new(Mutex::new(Some(writer0)));

    let writer_2 = writer.clone();

    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };

    stream.0 = device
        .build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<f32, f32>(data, &writer_2),
            err_fn,
            None,
        )
        .expect("stream fail");

    stream.0.play().expect("failed to play stream");

    Ok(())
}

fn str_from_path(path: PathBuf) -> String {
    let x = path.clone();
    let x2 = x.into_os_string().clone();
    let x3 = x2.into_string().unwrap();

    x3[4..x3.len()].to_string()
}

#[tauri::command]
fn get_wav_data(path: &str, app_handle: tauri::AppHandle) -> Result<(Vec<f32>, Vec<f32>), &str> {
    let mut v = vec![];
    let mut vfft: Vec<f32> = vec![];

    let p = app_handle
        .path_resolver()
        .resolve_resource(ASSETS_PATH)
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    if let Ok(mut reader) = hound::WavReader::open(p + "/" + path) {
        let itr = reader.samples::<f32>().into_iter().step_by(1);
        let mut buffer = vec![];
        let len = itr.len();
        for s in itr {
            let x = s.unwrap() as f32;
            v.push(x.clone());
            buffer.push(Complex { re: x, im: 0.0f32 })
        }
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(len);

        fft.process(&mut buffer);

        let mut vfft = vec![];
        for i in buffer {
            vfft.push(i.norm());
        }
        return Ok((v, vfft));
    } else {
        return Err("bad path");
    }

    Ok((v, vfft))
}

#[tauri::command]
fn get_stft_data(
    path: &str,
    app_handle: tauri::AppHandle,
) -> Result<(Vec<f32>, Vec<Vec<f32>>), &str> {
    let mut v = vec![];
    let mut vstft: Vec<Vec<f32>> = vec![];

    let p = app_handle
        .path_resolver()
        .resolve_resource(ASSETS_PATH)
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    if let Ok(mut reader) = hound::WavReader::open(p + "/" + path) {
        let itr = reader.samples::<f32>().into_iter().step_by(1);
        let mut buffer = vec![];
        let len = itr.len();
        for s in itr {
            let x = s.unwrap() as f32;
            v.push(x.clone());
            buffer.push(Complex { re: x, im: 0.0f32 })
        }

        let fftsize = 512;
        let vstft = stft(buffer.clone(), fftsize, fftsize);

        return Ok((v, vstft));
    } else {
        return Err("bad path");
    }

    Ok((v, vstft))
}

fn stft(mut buffer: Vec<Complex<f32>>, size: usize, hop: usize) -> Vec<Vec<f32>> {
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(size);

    let l = buffer.len();
    let num_slices = (l / size);
    let mut spectra: Vec<Vec<f32>> = vec![];
    for slice in 0..num_slices {
        let mut x = buffer[slice * size..(slice + 1) * size].to_vec().clone();

        fft.process(&mut x);

        let mut v = vec![];
        for i in x {
            v.push(i.norm());
        }
        spectra.push(v);
    }
    println!("{:?}", spectra.len());

    spectra
}
