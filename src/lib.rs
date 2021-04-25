extern crate rand;
extern crate vst;

use rand::Rng;
use std::fmt;
use vst::buffer::AudioBuffer;
use vst::plugin::{Plugin, Info};

pub struct Karplus {
    frequency: f32,
    sample_rate: u32,
    buffer: Vec<f32>
}

impl Karplus {
    pub fn new(frequency: f32, sample_rate: u32) -> Karplus {
        let size = (sample_rate as f32 / frequency) as usize;
        let mut v: Vec<f32> = Vec::with_capacity(size);
        let mut r = rand::thread_rng();
        for _ in 0..size {
            v.push((r.gen::<f32>() * 2.0) -1.0);
        }
        Karplus{
            frequency: frequency,
            sample_rate: sample_rate,
            buffer: v
        }
    }

    pub fn sample(&mut self, damping: f32) -> f32 {
        let v: f32 = self.buffer.remove(0);
        let s: f32 = (v + self.buffer[0])*0.5 * damping;

        self.buffer.push(s);
        s
    }
}

impl fmt::Display for Karplus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.frequency, self.sample_rate)
    }
}

impl Default for Karplus {
    fn default() -> Self {
        Karplus {
            frequency: 440.0,
            sample_rate: 44100
        }
    }
}


impl Plugin for Karplus {
    fn get_info(&self) -> Info {
        Info {
            name: "UQLRF".to_string(),
            vendor: "strikles".to_string(),
            unique_id: 999,

            inputs: 2,
            outputs: 2,
            parameters: 2,

            ..Info::default()
        }
    }

    fn get_parameter_label(&self, index: i32) -> String {
        match index {
            0 => "%".to_string(),
            _ => "".to_string(),
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("{}", self.frequency),
            _ => "".to_string(),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Frequency".to_string(),
            _ => "".to_string(),
        }

    }

    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.frequency,
            _ => 0.0,
        }
    }

    fn set_parameter(&mut self, index: i32, value: f32) {
        match index {
            0 => self.frequency = value.max(0.01),
            _ => (),
        };
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        for (input, output) in buffer.zip() {
            // For each input sample and output sample in buffer
            for (in_frame, out_frame) in input.into_iter().zip(output.into_iter()) {
                if *in_frame >= 0.0 {
                    let k = Karplus::new(self.frequency, self.sample_rate);
                    *out_frame = in_frame.min(self.frequency)
                }
            }
        }
    }
}

plugin_main!(Karplus);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_karplus_new() {
        let k = Karplus::new(440.0, 4400);
        assert_eq!(k.frequency, 440.0);
        assert_eq!(k.sample_rate, 4400);
        assert_eq!(k.buffer.len(), 10);
    }

    #[test]
    fn test_karplus_sample() {
        let mut k = Karplus::new(440.0, 44100);
        k.buffer = vec![1.0, 0.0];
        let sample = k.sample(1.0);
        assert_eq!(sample, 0.5)
    }
}
