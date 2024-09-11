// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused)]

use chrono::{self, DateTime, Utc};
use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SizedSample, Stream, StreamConfig};
use std::f32::consts::{PI, SQRT_2};
use std::fs::{self, Metadata};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::SystemTime;

use rustfft::{num_complex::Complex, FftPlanner};
use std::sync::Mutex;
use std::{fs::metadata, fs::File, io::BufWriter, process::Command};
use tauri::command::CommandItem;
use tauri::{LogicalSize, State};
use tauri::{Manager, Window};

mod audio;
use audio::*;
mod butterworth;
use butterworth::*;

fn main() {
    tauri::Builder::default()
        .manage(Mbool(Default::default()))
        .manage(AlwaysOnTop(Default::default()))
        .manage(Mstream({
            let host = cpal::default_host();

            let device = host
                .default_input_device()
                .expect("failed to get default audio device");

            let config = device
                .default_input_config()
                .expect("Failed to get default input config");

            let spec = wav_spec_from_config(&config);
            let writer0 = hound::WavWriter::create(".blank.wav", spec).expect("writer failed");
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
            let host = cpal::default_host();
            let device = host
                .default_input_device()
                .expect("failed to get default audio device");

            let config = device
                .default_input_config()
                .expect("Failed to get default input config");

            let spec = wav_spec_from_config(&config);

            let writer0 = hound::WavWriter::create(".blank.wav", spec).expect("writer failed");
            let writer = Arc::new(Mutex::new(Some(writer0)));

            Mutex::new(writer)
        }))
        .setup(|app| {
            let window = app.get_window("main").unwrap();

            #[cfg(target_os = "macos")]
            window.set_size(LogicalSize {
                width: 950,
                height: 470,
            });

            #[cfg(not(target_os = "macos"))]
            window.set_size(LogicalSize {
                width: 950,
                height: 450,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            record,
            get_wavs,
            stop_recording,
            file_metadata,
            get_wav_data,
            get_stft_data,
            rename_file,
            play,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
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
fn stop_recording(
    name: &str,
    trim: bool,
    app_handle: tauri::AppHandle,
    is_recording: State<Mbool>,
    mwriter: State<Mwriter>,
    mstream: State<Mstream>,
) -> Result<(), String> {
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

    if trim {
        let p = app_handle
            .path_resolver()
            .resource_dir()
            .unwrap()
            .join("assets")
            .join(name);

        let r = hound::WavReader::open(p.clone());
        let reader = if r.is_ok() {
            r.unwrap()
        } else {
            return Err("failed to read wav file".to_string());
        };
        let spec = reader.spec();
        let samples: Vec<f32> = reader.into_samples::<f32>().map(|x| x.unwrap()).collect();
        let mut writer = hound::WavWriter::create(p, spec).unwrap();
        let t1 = (spec.sample_rate as f32 * 0.1) as usize;
        let t2 = samples.len() - (spec.sample_rate as f32 * 0.3) as usize;
        println!("{:?}", t1);
        println!("{:?}", t2);

        for (i, samp) in samples.iter().enumerate() {
            if i < t2 {
                writer.write_sample(*samp);
            }
        }

        writer.finalize().expect("failed to finalize trim writer");
    }

    Ok(())
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
fn get_wavs(app_handle: tauri::AppHandle) -> Vec<String> {
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to get resource dir")
        .join("assets");

    let files = fs::read_dir(p).unwrap();
    let mut strings = vec![];
    for file in files.into_iter() {
        strings.push(file.unwrap().file_name().into_string().unwrap());
    }
    strings
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

    let host = cpal::default_host();

    let device = host
        .default_input_device()
        .expect("failed to get default audio device");

    let config = device
        .default_input_config()
        .expect("Failed to get default input config");

    let input_path = app_handle
        .path_resolver()
        .resource_dir()
        .unwrap()
        .join("assets")
        .join(name);

    let spec = wav_spec_from_config(&config);
    let writer0 = hound::WavWriter::create(input_path, spec).expect("writer failed");
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
fn get_wav_data(
    file_name: &str,
    app_handle: tauri::AppHandle,
) -> Result<(Vec<f32>, Vec<f32>), &str> {
    let mut v = vec![];
    let mut vfft: Vec<f32> = vec![];

    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .join("assets")
        .join(file_name);

    if let Ok(mut reader) = hound::WavReader::open(p) {
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
        for i in buffer[0..len / 2].iter() {
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
    file_name: &str,
    app_handle: tauri::AppHandle,
) -> Result<(Vec<f32>, Vec<f32>), &str> {
    let mut v = vec![];
    let mut vstft: Vec<f32> = vec![];

    let input_path = app_handle
        .path_resolver()
        .resource_dir()
        .unwrap()
        .join("assets")
        .join(file_name);

    if let Ok(mut reader) = hound::WavReader::open(input_path) {
        let itr = reader.samples::<f32>().into_iter().step_by(1);
        let mut buffer = vec![];
        let len = itr.len();
        let fftsize = 1024;
        if len < fftsize {
            // return Err("file too short to get fft");
            return Ok((vec![], vec![]));
        }

        for s in itr {
            let x = s.unwrap() as f32;
            v.push(x.clone());
            buffer.push(Complex { re: x, im: 0.0f32 })
        }

        let mut vstft = diff_stft(&mut buffer, fftsize, fftsize);
        // let down_rate = v.len() / 600; // 600 is front-end width of plot
        // let o = v.iter().step_by(1).map(|x| *x).collect();

        return Ok((v, vstft[1..].to_vec()));
    } else {
        return Err("bad path");
    }

    Ok((v, vstft))
}

#[tauri::command]
fn rename_file(old: &str, new: &str, app_handle: tauri::AppHandle) -> Result<(), String> {
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .join("assets");

    let r = fs::rename(p.join(old), p.join(new));

    return r.map_err(|e| e.to_string());
}

#[tauri::command]
fn play(name: &str, app_handle: tauri::AppHandle) -> Result<(), String> {
    let host = cpal::default_host();

    let device = host.default_output_device().unwrap();

    let config = device.default_output_config().unwrap();
    match config.sample_format() {
        cpal::SampleFormat::I8 => {
            play_recording::<i8>(name, app_handle);
        }
        cpal::SampleFormat::I16 => {
            play_recording::<i16>(name, app_handle);
        }
        cpal::SampleFormat::I32 => {
            play_recording::<i32>(name, app_handle);
        }
        cpal::SampleFormat::I64 => {
            play_recording::<i64>(name, app_handle);
        }
        cpal::SampleFormat::U8 => {
            play_recording::<u8>(name, app_handle);
        }
        cpal::SampleFormat::U16 => {
            play_recording::<u16>(name, app_handle);
        }
        cpal::SampleFormat::U32 => {
            play_recording::<u32>(name, app_handle);
        }
        cpal::SampleFormat::U64 => {
            play_recording::<u64>(name, app_handle);
        }
        cpal::SampleFormat::F32 => {
            play_recording::<f32>(name, app_handle);
        }
        cpal::SampleFormat::F64 => {
            play_recording::<f64>(name, app_handle);
        }
        // sample_format => Err(anyhow::Error::msg(format!(
        //     "Unsupported sample format '{sample_format}'"
        // ))),
        _ => {}
    }

    Ok(())
}

fn play_recording<T>(name: &str, app_handle: tauri::AppHandle) -> Result<(), String>
where
    T: SizedSample + FromSample<f32>,
{
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .join("assets")
        .join(name);

    let r = hound::WavReader::open(p.clone());
    let reader = if r.is_ok() {
        r.unwrap()
    } else {
        return Err("derp".to_string());
    };
    let spec = reader.spec();
    let samp_iter = reader.into_samples();
    let mut samples: Vec<f32> = vec![];
    for samp in samp_iter {
        samples.push(samp.unwrap());
    }

    let host = cpal::default_host();
    let err_fn = |err| eprintln!("Error building output sound stream: {}", err);

    let device = host.default_output_device().unwrap();

    let conf: StreamConfig = device.default_output_config().unwrap().into();
    let num_channels = conf.channels as usize;
    let num_samples = samples.len();

    tauri::async_runtime::spawn(async move {
        let sleep = samples.len() as f32 / conf.sample_rate.0 as f32;
        let mut t = 0;
        let window = app_handle.get_window("main").unwrap();
        let stream = device
            .build_output_stream(
                &conf.into(),
                move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
                    for frame in output.chunks_mut(num_channels) {
                        if t >= num_samples {
                            break;
                        }
                        // assuming mono input, will write to both outputs if output is stereo
                        let v: T = T::from_sample(samples[t]);
                        for out_sample in frame.iter_mut() {
                            *out_sample = v;
                        }
                        t += 1;
                        window.emit("time_indicator", { t });
                    }
                },
                err_fn,
                None,
            )
            .unwrap();

        let r = stream.play();

        std::thread::sleep(std::time::Duration::from_secs_f32(sleep));
        let window = app_handle.get_window("main").unwrap();
        window.emit("time_indicator", { 0 });
    });

    Ok(())
}

fn diff_stft(mut buffer: &Vec<Complex<f32>>, size: usize, hop: usize) -> Vec<f32> {
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(size);

    let l = buffer.len();
    let num_slices = (l / size);
    let mut spectra: Vec<f32> = vec![];
    let window = hamming_complex(size);
    let mut prev_spectrum = vec![CZERO; size];
    for slice in 0..num_slices {
        // let mut x = buffer[slice * size..(slice + 1) * size].to_vec();
        let mut x = vec![CZERO; size];
        for i in 0..size {
            x[i] = buffer[slice * size..(slice + 1) * size][i] * window[i]
        }

        fft.process(&mut x);
        let hpf = freq_hpf(25500.0, 44100.0, 1.0 / SQRT_2);

        for (i, cplx) in x[0..size / 2].iter().enumerate() {
            // spectra.push((cplx).norm_sqr());
            spectra.push((cplx).norm_sqr() - prev_spectrum[i].norm_sqr());
        }
        prev_spectrum = x.clone();
    }

    spectra
}

fn stft(mut buffer: &Vec<Complex<f32>>, size: usize, hop: usize) -> Vec<f32> {
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(size);

    let l = buffer.len();
    let num_slices = (l / size);
    let mut spectra: Vec<f32> = vec![];
    let window = hamming_complex(size);
    for slice in 0..num_slices {
        // let mut x = buffer[slice * size..(slice + 1) * size].to_vec();
        let mut x = vec![CZERO; size];
        for i in 0..size {
            x[i] = buffer[slice * size..(slice + 1) * size][i] * window[i]
        }

        fft.process(&mut x);
        let hpf = freq_hpf(200.0, 44100.0, 1.0 / SQRT_2);

        for (i, cplx) in x[0..size / 2].iter().enumerate() {
            spectra.push((hpf[i] * cplx).norm_sqr());
        }
    }

    spectra
}
