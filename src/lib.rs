#[macro_use]
extern crate vst;
extern crate rand;

use std::{convert::TryFrom, sync::Arc};

use vst::{
    api::{Events, Supported},
    buffer::AudioBuffer,
    editor::Editor,
    event::Event,
    plugin::{CanDo, Category, HostCallback, Info, Plugin, PluginParameters},
    util::AtomicFloat
};

use rand::random;

#[derive(Debug, Copy, Clone)]
struct Note {
    alpha: f64,
    note: u8,
    is_released: bool,
}

#[derive(Default)]
struct Karplus {
    params: Arc<KarplusParameters>,
    notes: Vec<Note>,
    sample_rate: f32,
    time: f64
}

#[derive(Default)]
struct KarplusParameters {
    frequency: AtomicFloat,
    gain: AtomicFloat,
    attack_duration: AtomicFloat,
    release_duration: AtomicFloat
    host: HostCallback
}

pub fn midi_pitch_to_freq(pitch: u8) -> f64 {
    const A4_PITCH: i8 = 69;
    const A4_FREQ: f64 = 440.0;

    // Midi notes can be 0-127
    ((f64::from(pitch as i8 - A4_PITCH)) / 12.).exp2() * A4_FREQ
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
            
            category: Category::Synth,

            ..Info::default()
        }
    }
    
    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
    
    fn new(host: HostCallback) -> Self {
        Karplus {
            params: Arc::new(KarplusParameters {
                host,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
    
    fn process_events(&mut self, events: &Events) {
        for event in events.events() {
            match event {
                Event::Midi(ev) => {
                    match ev.data[0] {
                        144 => {
                            self.notes.push(Note { note: ev.data[1], alpha: 0.0, is_released: false });
                        }
                        128 => {
                            for note in self.notes.iter_mut() {
                                if note.note == ev.data[1] {
                                    note.is_released = true;
                                }
                            }
                        }
                        _ => ()
                    }
                }

                _ => ()
            }
        }
    }

    fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate = rate;
    }
    
    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let samples = buffer.samples();
        let (_, mut outputs) = buffer.split();
        let output_count = outputs.len();

        let per_sample = (1.0 / self.sample_rate) as f64;
        let attack_per_sample = per_sample * (1.0 / self.params.attack_duration.get() as f64);
        let release_per_sample = per_sample * (1.0 / self.params.release_duration.get() as f64);


        let mut output_sample;
        for sample_idx in 0..samples {

            // Update the alpha of each note...
            for note in self.notes.iter_mut() {
                if !note.is_released && note.alpha < 1.0 {
                    note.alpha += attack_per_sample;
                }

                if note.is_released {
                    note.alpha -= release_per_sample;
                }
            }

            // ...and remove finished notes.
            self.notes.retain(|n| n.alpha > 0.0);

            // Sum up all the different notes and noise types
            if !self.notes.is_empty() {
                let mut signal = 0.0;
                let params = self.params.deref();

                for note in &self.notes {
                    let point = [0.0, self.time * midi_pitch_to_freq(note.note)];

                    if note.alpha > 0.0001 {
                        signal += ((random::<f64>() - 0.5) * 2.0) as f64 * note.alpha;
                    }
                }

                output_sample = signal as f32;
                self.time += per_sample;
                
            } else {
                output_sample = 0.0;
            }

            for buf_idx in 0..output_count {
                let buff = outputs.get_mut(buf_idx);
                buff[sample_idx] = output_sample;
            }
        }
    }

    // It's good to tell our host what our plugin can do.
    // Some VST hosts might not send any midi events to our plugin
    // if we don't explicitly tell them that the plugin can handle them.
    fn can_do(&self, can_do: CanDo) -> Supported {
        match can_do {
            // Tell our host that the plugin supports receiving MIDI messages
            CanDo::ReceiveMidiEvent => Supported::Yes,
            // Maybe it also supports ather things
            _ => Supported::Maybe,
        }
    }

    /*
    fn new(frequency: f32, sample_rate: u32) -> Karplus {
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

    fn process(&mut self, damping: f32) -> f32 {
        let v: f32 = self.buffer.remove(0);
        let s: f32 = (v + self.buffer[0])*0.5 * damping;

        self.buffer.push(s);
        s
    }
    */
}

impl PluginParameters for KarplusParameters {
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.frequency.get(),
            1 => self.gain.get(),
            _ => 0.0,
        }
    }

    fn set_parameter(&self, index: i32, value: f32) {
        #[allow(clippy::single_match)]
        match index {
            0 => self.frequency.set(value.max(0.01)),
            1 => self.gain.set(value),
            _ => (),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Frequency".to_string(),
            1 => "Gain".to_string(),
            _ => "".to_string(),
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("{:.2}", self.frequency.get()),
            1 => format!("{:.2}", self.gain.get()),
            _ => "".to_string(),
        }
    }
}

plugin_main!(Karplus);

/*
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
*/
