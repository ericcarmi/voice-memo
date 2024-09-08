use std::f32::consts::PI;

use rustfft::num_complex::Complex;

pub fn lowpass(frequency: f32, sample_rate: f32, q: f32) -> Vec<Complex<f32>> {
    let w0 = 2.0 * PI * frequency / sample_rate;
    let sin = w0.sin();
    let cos = w0.cos();
    let alpha = sin / (2.0 * q);

    let mut iir = IIR2::new();

    iir.b0 = (1.0 - cos) / 2.0;
    iir.b1 = 1.0 - cos;

    iir.a0 = 1.0 + alpha;
    iir.a1 = -2.0 * cos;
    iir.a2 = 1.0 - alpha;

    iir.freq_response(1024)
}

// generate a frequency domain lowpass filter
pub fn freq_hpf(frequency: f32, sample_rate: f32, q: f32) -> Vec<Complex<f32>> {
    let w0 = 2.0 * PI * frequency / sample_rate;
    let sin = w0.sin();
    let cos = w0.cos();
    let alpha = sin / (2.0 * q);

    let mut iir = IIR2::new();

    iir.b0 = (1.0 + cos) / 2.0;
    iir.b1 = -1.0 - cos;

    iir.a0 = 1.0 + alpha;
    iir.a1 = -2.0 * cos;
    iir.a2 = 1.0 - alpha;

    iir.freq_response(1024)
}

#[derive(Clone, Copy, Debug)]
pub struct IIR2 {
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub a0: f32,
    pub a1: f32,
    pub a2: f32,
    pub x: [f32; 2],
    pub y: [f32; 2],
}

impl From<BPF> for IIR2 {
    fn from(bpf: BPF) -> Self {
        let g = (bpf.gain / 40.0).powf(10.0);
        let w0 = (2.0 * PI * bpf.freq) / SAMPLING_RATE;
        let alpha = (w0).sin() / 2.0 / bpf.Q;
        Self {
            b0: 1.0 + alpha * g,
            b1: -2.0 * w0.cos(),
            b2: 1.0 - alpha * g,
            a0: 1.0 + alpha / g,
            a1: -2.0 * w0.cos(),
            a2: 1.0 - alpha / g,
            x: [0.0, 0.0],
            y: [0.0, 0.0],
        }
    }
}

impl IIR2 {
    pub fn new() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a0: 1.0,
            a1: 0.0,
            a2: 0.0,
            x: [0.0, 0.0],
            y: [0.0, 0.0],
        }
    }
    pub fn update_coeffs(&mut self, iir: IIR2) {
        self.b0 = iir.b0;
        self.b1 = iir.b1;
        self.b2 = iir.b2;
        self.a0 = iir.a0;
        self.a1 = iir.a1;
        self.a2 = iir.a2;
    }

    pub fn freq_response(&self, n: usize) -> Vec<Complex<f32>> {
        let mut H = vec![];
        let L = n as f32;
        for i in 0..n {
            let x = (-PI * i as f32 / L).cos();
            let y = (-PI * i as f32 / L).sin();
            let z = Complex { re: x, im: y };
            let z2 = z * z;

            let w = (self.b0 + self.b1 * z + self.b2 * z2) / (self.a0 + self.a1 * z + self.a2 * z2);

            H.push(w);
        }

        H
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BPF {
    pub gain: f32,
    pub freq: f32,
    pub Q: f32,
}

impl BPF {
    pub fn new() -> Self {
        Self {
            gain: 0.0,
            freq: 1000.0,
            Q: 1.0,
        }
    }
}

pub const SAMPLING_RATE: f32 = 44100.0;
