#[macro_use]
extern crate vst;
extern crate rand;
extern crate noise;

use std::{sync::Arc, ops::Deref};

use vst::{
    api::{Events, Supported},
    buffer::AudioBuffer,
    //editor::Editor,
    event::Event,
    plugin::{CanDo, Category, HostCallback, Info, Plugin, PluginParameters},
    util::AtomicFloat
};

use noise::{NoiseFn, Perlin, Worley, Billow, Cylinders, OpenSimplex, RidgedMulti, Value, HybridMulti, BasicMulti};

use rand::random;

#[derive(Debug, Copy, Clone)]
struct Note {
    alpha: f64,
    note: u8,
    is_released: bool,
}

struct Karplus {
    params: Arc<KarplusParameters>,
    notes: Vec<Note>,
    sample_rate: f32,
    time: f64,
    // Noise functions
    fn_perlin: Perlin,
    fn_value: Value,
    fn_worley: Worley,
    fn_ridged_multi: RidgedMulti,
    fn_open_simplex: OpenSimplex,
    fn_billow: Billow,
    fn_cylinders: Cylinders,
    fn_hybrid_multi: HybridMulti,
    fn_basic_multi: BasicMulti,
}

pub struct KarplusParameters {
    // Amounts
    pub a_white_noise: AtomicFloat,
    pub a_perlin: AtomicFloat,
    pub a_value: AtomicFloat,
    pub a_worley: AtomicFloat,
    pub a_ridged_multi: AtomicFloat,
    pub a_open_simplex: AtomicFloat,
    pub a_billow: AtomicFloat,
    pub a_cylinders: AtomicFloat,
    pub a_hybrid_multi: AtomicFloat,
    pub a_basic_multi: AtomicFloat,
    pub attack_duration: AtomicFloat,
    pub release_duration: AtomicFloat,
    pub damping: AtomicFloat,
    pub host: HostCallback
}

pub fn midi_pitch_to_freq(pitch: u8) -> f64 {
    const A4_PITCH: i8 = 69;
    const A4_FREQ: f64 = 440.0;

    // Midi notes can be 0-127
    ((f64::from(pitch as i8 - A4_PITCH)) / 12.).exp2() * A4_FREQ
}

impl Default for Karplus {
    fn default() -> Karplus {
        Karplus {
            notes: vec![],
            sample_rate: 44100.0,
            time: 0.0,
            // Amounts
            params: Arc::new(KarplusParameters::default()),

            // Noise functions
            fn_perlin: Perlin::new(),
            fn_value: Value::new(),
            fn_worley: Worley::new(),
            fn_ridged_multi: RidgedMulti::new(),
            fn_open_simplex: OpenSimplex::new(),
            fn_billow: Billow::new(),
            fn_cylinders: Cylinders::new(),
            fn_hybrid_multi: HybridMulti::new(),
            fn_basic_multi: BasicMulti::new(),
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
            parameters: 13,
            
            category: Category::Synth,

            ..Info::default()
        }
    }
    
    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
    
    fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate = rate;
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

                    if params.a_white_noise.get() > 0.0 && note.alpha > 0.0001 {
                        signal += ((random::<f64>() - 0.5) * 2.0) * params.a_white_noise.get() as f64 * note.alpha;
                    }

                    if params.a_perlin.get() > 0.0 && note.alpha > 0.0001 {
                        signal += self.fn_perlin.get(point) * params.a_perlin.get() as f64 * note.alpha;
                    }

                    if params.a_value.get() > 0.0 && note.alpha > 0.0001 {
                        signal += self.fn_value.get(point) * params.a_value.get() as f64 * note.alpha;
                    }

                    if params.a_worley.get() > 0.0 && note.alpha > 0.0001 {
                        signal += self.fn_worley.get(point) * params.a_worley.get() as f64 * note.alpha;
                    }

                    if params.a_ridged_multi.get() > 0.0 && note.alpha > 0.0001 {
                        signal += self.fn_ridged_multi.get(point) * params.a_ridged_multi.get() as f64 * note.alpha;
                    }

                    if params.a_open_simplex.get() > 0.0 && note.alpha > 0.0001 {
                        signal += self.fn_open_simplex.get(point) * params.a_open_simplex.get() as f64 * note.alpha;
                    }

                    if params.a_billow.get() > 0.0 && note.alpha > 0.0001 {
                        signal += self.fn_billow.get(point) * params.a_billow.get() as f64 * note.alpha;
                    }

                    if params.a_cylinders.get() > 0.0 && note.alpha > 0.0001 {
                        signal += self.fn_cylinders.get(point) * params.a_cylinders.get() as f64 * note.alpha;
                    }

                    if params.a_hybrid_multi.get() > 0.0 && note.alpha > 0.0001 {
                        signal += self.fn_hybrid_multi.get(point) * params.a_hybrid_multi.get() as f64 * note.alpha;
                    }

                    if params.a_basic_multi.get() > 0.0 && note.alpha > 0.0001 {
                        signal += self.fn_basic_multi.get(point) * params.a_basic_multi.get() as f64 * note.alpha;
                    }
                }

                output_sample = signal as f32;
                self.time += per_sample;
            } else {
                output_sample = 0.0;
            }

            // copy noise to buff
            for buf_idx in 0..output_count {
                let buff = outputs.get_mut(buf_idx);
                buff[sample_idx] = output_sample;
            }
        }
        for sample_idx in 0..samples {
            // hmmm
            if !self.notes.is_empty() {
                for note in &self.notes {
                    //let num = self.sample_rate / midi_pitch_to_freq(note.note);
                }
            }
            // ks
            for buf_idx in 0..output_count {
                let buff = outputs.get_mut(buf_idx);
                let n = 200;
                buff[sample_idx] = 0.5 * self.params.damping.get() * (buff[sample_idx % n] + buff[(sample_idx + 1) % n]);
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
}


impl PluginParameters for KarplusParameters {
    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("{:.1}%", self.a_white_noise.get() * 100.0),
            1 => format!("{:.1}%", self.a_perlin.get() * 100.0),
            2 => format!("{:.1}%", self.a_value.get() * 100.0),
            3 => format!("{:.1}%", self.a_worley.get() * 100.0),
            4 => format!("{:.1}%", self.a_ridged_multi.get() * 100.0),
            5 => format!("{:.1}%", self.a_open_simplex.get() * 100.0),
            6 => format!("{:.1}%", self.a_billow.get() * 100.0),
            7 => format!("{:.1}%", self.a_cylinders.get() * 100.0),
            8 => format!("{:.1}%", self.a_hybrid_multi.get() * 100.0),
            9 => format!("{:.1}%", self.a_basic_multi.get() * 100.0),
            10 => format!("{:.1}s", self.attack_duration.get()),
            11 => format!("{:.1}s", self.release_duration.get()),
            12 => format!("{:.1}s", self.damping.get()),
            _ => "".to_string(),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "White",
            1 => "Perlin",
            2 => "Value",
            3 => "Worley",
            4 => "RidgedMulti",
            5 => "OpenSimplex",
            6 => "Billow",
            7 => "Cylinders",
            8 => "HybridMulti",
            9 => "BasicMulti",
            10 => "Attack",
            11 => "Release",
            12 => "Damping",
            _ => "",
        }.to_string()
    }

    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.a_white_noise.get(),
            1 => self.a_perlin.get(),
            2 => self.a_value.get(),
            3 => self.a_worley.get(),
            4 => self.a_ridged_multi.get(),
            5 => self.a_open_simplex.get(),
            6 => self.a_billow.get(),
            7 => self.a_cylinders.get(),
            8 => self.a_hybrid_multi.get(),
            9 => self.a_basic_multi.get(),
            10 => self.attack_duration.get(),
            11 => self.release_duration.get(),
            12 => self.damping.get(),
            _ => 0.0,
        }
    }

    fn set_parameter(&self, index: i32, val: f32) {
        match index {
            0 => self.a_white_noise.set(val),
            1 => self.a_perlin.set(val),
            2 => self.a_value.set(val),
            3 => self.a_worley.set(val),
            4 => self.a_ridged_multi.set(val),
            5 => self.a_open_simplex.set(val),
            6 => self.a_billow.set(val),
            7 => self.a_cylinders.set(val),
            8 => self.a_hybrid_multi.set(val),
            9 => self.a_basic_multi.set(val),
            10 => self.attack_duration.set(val.max(0.001)), // prevent division by zero
            11 => self.release_duration.set(val.max(0.001)),
            12 => self.damping.set(val),
            _ => (),
        }
    }
}

impl Default for KarplusParameters {
    fn default() -> KarplusParameters {
        KarplusParameters {
            a_white_noise: AtomicFloat::new(1.0),
            a_perlin: AtomicFloat::new(0.0),
            a_value: AtomicFloat::new(0.0),
            a_worley: AtomicFloat::new(0.0),
            a_ridged_multi: AtomicFloat::new(0.0),
            a_open_simplex: AtomicFloat::new(0.0),
            a_billow: AtomicFloat::new(0.0),
            a_cylinders: AtomicFloat::new(0.0),
            a_hybrid_multi: AtomicFloat::new(0.0),
            a_basic_multi: AtomicFloat::new(0.0),
            attack_duration: AtomicFloat::new(0.5),
            release_duration: AtomicFloat::new(0.5),
            damping: AtomicFloat::new(0.996),
            host:   HostCallback::default(),
        }
    }
}

plugin_main!(Karplus);
