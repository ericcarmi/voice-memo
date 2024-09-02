use anyhow;
use cpal::{self};
use cpal::{
    traits::{DeviceTrait, HostTrait},
    SizedSample,
};
use cpal::{FromSample, Stream};
use rustfft::num_complex::Complex;
use std::f32::consts::PI;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Window};

use crate::WavWriterHandle;

fn get_wav_samples(path: PathBuf, app_handle: AppHandle) -> Result<(Vec<f32>, bool), String> {
    if let Ok(file_in) = File::open(path) {
        let (head, samples) = wav_io::read_from_file(file_in).unwrap();
        let is_stereo = if head.channels == 1 { false } else { true };
        Ok((samples, is_stereo))
    } else {
        return Err("failed to get wav sampels".to_string());
    }
}

#[derive(Clone, Debug, Default)]
pub struct AudioUIMessage {}

#[derive(Clone, Debug)]
pub struct UIAudioMessage {}

// cpal stream
pub struct MStream(pub Mutex<Stream>);
// receive message for ui
pub struct MUIReceiver(pub Mutex<tauri::async_runtime::Receiver<AudioUIMessage>>);
// send message from audio to ui
pub struct MAudioSender(pub Mutex<tauri::async_runtime::Sender<AudioUIMessage>>);
// send message from ui to audio thread
pub struct MSender(pub Mutex<tauri::async_runtime::Sender<UIAudioMessage>>);
pub struct MStreamSend(pub Mutex<StreamSend>);

pub struct StreamSend {
    pub stream: MStream,
    pub msender: MSender,
    pub mreceiver: MUIReceiver,
    pub mtx_ui: MAudioSender,
}
unsafe impl Sync for MStream {}
unsafe impl Send for MStream {}
unsafe impl Send for MSender {}
unsafe impl Sync for MSender {}
unsafe impl Send for MUIReceiver {}
unsafe impl Sync for MUIReceiver {}
unsafe impl Send for MAudioSender {}
unsafe impl Sync for MAudioSender {}
unsafe impl Send for MStreamSend {}
unsafe impl Sync for MStreamSend {}
unsafe impl Send for StreamSend {}
unsafe impl Sync for StreamSend {}
unsafe impl Send for AudioUIMessage {}
unsafe impl Sync for AudioUIMessage {}

pub fn hamming(n: usize) -> Vec<f32> {
    (0..n)
        .map(|i| (0.54 - 0.46 * (2.0 * PI * (i as f32) / (n as f32)).cos()))
        .collect()
}
pub fn hamming_complex(n: usize) -> Vec<Complex<f32>> {
    (0..n)
        .map(|i| Complex {
            re: (0.54 - 0.46 * (2.0 * PI * (i as f32) / (n as f32)).cos()),
            im: 0.0,
        })
        .collect()
}

pub struct WavMetadata {
    created: &'static str,
    sample_rate: i32,
}

#[derive(Debug)]
pub struct Mbool(pub Mutex<bool>);
pub struct AlwaysOnTop(pub Mutex<bool>);
pub struct Mwriter(pub Mutex<WavWriterHandle>);
pub struct NStream(pub Stream);
pub struct Mstream(pub Mutex<NStream>);
pub struct MOutStream(pub Mutex<NStream>);
pub const CZERO: Complex<f32> = Complex { re: 0.0, im: 0.0 };
unsafe impl std::marker::Send for NStream {}
