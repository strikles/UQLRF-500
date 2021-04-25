#[macro_use]
extern crate vst;
extern crate rand;

use vst::plugin::{Info, Plugin, Category, CanDo};
use vst::buffer::AudioBuffer;
use vst::event::Event;
use vst::api::{Events, Supported};
use rand::random;

#[derive(Default)]
struct Karplus {
    frequency: f32,
    sample_rate: u32,
    buffer: Vec<f32>
}

/*
impl fmt::Display for Karplus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.frequency, self.sample_rate)
    }
}
*/

impl Default for Karplus {
    
    fn default() -> Self {
        Karplus {
            frequency: 440.0,
            sample_rate: 44190,
            buffer: vec![1.0, 0.0]
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
            parameters: 1,

            ..Info::default()
        }
    }
    
    // Here's the function that allows us to receive events
    fn process_events(&mut self, events: &Events) {

        // Some events aren't MIDI events - so let's do a match
        // to make sure we only get MIDI, since that's all we care about.
        for event in events.events() {
            match event {
                Event::Midi(ev) => {

                    // Check if it's a noteon or noteoff event.
                    // This is difficult to explain without knowing how the MIDI standard works.
                    // Basically, the first byte of data tells us if this signal is a note on event
                    // or a note off event.  You can read more about that here: 
                    // https://www.midi.org/specifications/item/table-1-summary-of-midi-message
                    match ev.data[0] {

                        // if note on, increment our counter
                        144 => self.notes += 1u8,

                        // if note off, decrement our counter
                        128 => self.notes -= 1u8,
                        _ => (),
                    }
                    // if we cared about the pitch of the note, it's stored in `ev.data[1]`.
                },
                // We don't care if we get any other type of event
                _ => (),
            }
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {

        // `buffer.split()` gives us a tuple containing the 
        // input and output buffers.  We only care about the
        // output, so we can ignore the input by using `_`.
        let (_, output_buffer) = buffer.split();

        // We only want to process *anything* if a note is being held.
        // Else, we can fill the output buffer with silence.
        if self.notes == 0 {
            for output_channel in output_buffer.into_iter() {
                // Let's iterate over every sample in our channel.
                for output_sample in output_channel {
                    *output_sample = 0.0;
                }
            }
            return;
        }

        // Now, we want to loop over our output channels.  This
        // includes our left and right channels (or more, if you
        // are working with surround sound).
        for output_channel in output_buffer.into_iter() {
            // Let's iterate over every sample in our channel.
            for output_sample in output_channel {
                // For every sample, we want to generate a random value
                // from -1.0 to 1.0.
                *output_sample = (random::<f32>() - 0.5f32) * 2f32;
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

impl PluginParameters for Karplus {
    
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.frequency,
            _ => 0.0,
        }
    }

    fn set_parameter(&self, index: i32, value: f32) {
        //        match index {
        //            // We don't want to divide by zero, so we'll clamp the value
        //            0 => self.threshold = value.max(0.01),
        //            _ => (),
        //        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Frequency".to_string(),
            _ => "".to_string(),
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            // Convert to a percentage
            0 => format!("{}", self.frequency),
            _ => "".to_string(),
        }
    }

    fn get_parameter_label(&self, index: i32) -> String {
        match index {
            0 => "%".to_string(),
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
