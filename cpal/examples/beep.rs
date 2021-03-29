extern crate anyhow;
extern crate clap;
extern crate cpal;
extern crate dsp;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use dsp::{Frame, FromSample, Graph, Node, Sample};

#[derive(Debug)]
struct Opt {
    #[cfg(all(
        any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd"),
        feature = "jack"
    ))]
    jack: bool,

    device: String,
}


const CHANNELS: usize = 2;
// const FRAMES: u32 = 64;
const SAMPLE_HZ: f64 = 44_100.0;


type Output = f32;

type Phase = f64;
type Frequency = f64;
type Volume = f32;


const A5_HZ: Frequency = 40.0;
const D5_HZ: Frequency = 587.33;
const F5_HZ: Frequency = 900.46;

impl Opt {
    fn from_args() -> Self {
        let app = clap::App::new("beep").arg_from_usage("[DEVICE] 'The audio device to use'");
        #[cfg(all(
            any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd"),
            feature = "jack"
        ))]
        let app = app.arg_from_usage("-j, --jack 'Use the JACK host");
        let matches = app.get_matches();
        let device = matches.value_of("DEVICE").unwrap_or("default").to_string();

        #[cfg(all(
            any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd"),
            feature = "jack"
        ))]
        return Opt {
            jack: matches.is_present("jack"),
            device,
        };

        #[cfg(any(
            not(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd")),
            not(feature = "jack")
        ))]
        Opt { device }
    }
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();

    let host = cpal::default_host();

    let device = if opt.device == "default" {
        host.default_output_device()
    }else {
        host.output_devices()?
            .find(|x| x.name().map(|y| y == opt.device).unwrap_or(false))

    }
    .expect("failed to find output device");
    println!("Output device: {}", device.name()?);

    let config = device.default_output_config().unwrap();
    println!("Default output config: {:?}", config);

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()),
    }
}

pub fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), anyhow::Error>
where
    T: cpal::Sample,
{


    // let mut graph = Graph::new();

    // // Construct our fancy Synth and add it to the graph!
    // let synth = graph.add_node(DspNode::Synth(0.0));

    // // Output our synth to a marvellous volume node.
    // let (_, volume) = graph.add_output(synth, DspNode::Volume(1.0));

    // // Set the synth as the master node for the graph.
    // graph.set_master(Some(volume));


    let mut graph = Graph::new();

    // Construct our fancy Synth and add it to the graph!
    let synth = graph.add_node(DspNode::Synth);

    // Connect a few oscillators to the synth.
    graph.add_input(DspNode::Oscillator(0.0, A5_HZ, 0.2), synth);
    graph.add_input(DspNode::Oscillator(0.0, D5_HZ, 0.1), synth);
    graph.add_input(DspNode::Oscillator(0.0, F5_HZ, 0.15), synth);

    // If adding a connection between two nodes would create a cycle, Graph will return an Err.
    // if let Err(err) = graph.add_connection(synth, oscillator_a) {
    //     println!(
    //         "Testing for cycle error: {:?}",
    //         std::error::Error::description(&err)
    //     );
    // }

    // Set the synth as the master node for the graph.
    graph.set_master(Some(synth));




    // We'll use this to count down from three seconds and then break from the loop.
    // let timer: f64 = 0.0;

    // This will be used to determine the delta time between calls to the callback.
    // let mut prev_time = None;




    // let sample_rate = config.sample_rate.0 as f32;
    // let channels = config.channels as usize;

    // Produce a sinusoid of maximum amplitude.
    // let mut sample_clock = 0f32;
    // let mut next_value = move || {
    //     sample_clock = (sample_clock + 1.0) % sample_rate;
    //     (sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sample_rate).sin()
    // };

    

    // let mut next_value = move |frame| {
    //     let buffer: &mut [[f32; CHANNELS]] = frame;
    //     dsp::slice::equilibrium(buffer);
    //     graph.audio_requested(buffer, SAMPLE_HZ);
    //     buffer[0][0]
    // };

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(1) {
                
                let buffer: &mut [[f32; CHANNELS]] = &mut [[0f32, 0f32]];

                dsp::slice::equilibrium(buffer);
                graph.audio_requested(buffer, SAMPLE_HZ);

                // Traverse inputs or outputs of a node with the following pattern.
                // let mut inputs = graph.inputs(synth);
                // while let Some(input_idx) = inputs.next_node(&graph) {
                //     if let DspNode::Oscillator(_, ref mut pitch, _) = graph[input_idx] {
                //         // Pitch down our oscillators for fun.
                //         *pitch -= 0.1;
                //     }
                // }                

                for (i, sample) in frame.iter_mut().enumerate() {
                    *sample = cpal::Sample::from::<f32>(&buffer[0][i]);
                }
            }



        },
        err_fn,
    )?;
    stream.play()?;

    std::thread::sleep(std::time::Duration::from_millis(5000));

    Ok(())
}


#[derive(Debug)]
enum DspNode {
    /// Synth will be our demonstration of a master GraphNode.
    Synth,
    /// Oscillator will be our generator type of node, meaning that we will override
    /// the way it provides audio via its `audio_requested` method.
    Oscillator(Phase, Frequency, Volume),
}

impl Node<[Output; CHANNELS]> for DspNode {
    /// Here we'll override the audio_requested method and generate a sine wave.
    fn audio_requested(&mut self, buffer: &mut [[Output; CHANNELS]], sample_hz: f64) {
        match *self {
            DspNode::Synth => (),
            DspNode::Oscillator(ref mut phase, frequency, volume) => {
                dsp::slice::map_in_place(buffer, |_| {
                    let val = sine_wave(*phase, volume);
                    *phase += frequency / sample_hz;
                    Frame::from_fn(|_| val)
                });
            }
        }
    }
}


// enum DspNode {
//     Synth(f64),
//     Volume(f32),
// }

// /// Implement the `Node` trait for our DspNode.
// impl Node<[f32; CHANNELS]> for DspNode {
//     fn audio_requested(&mut self, buffer: &mut [[f32; CHANNELS]], sample_hz: f64) {
//         match *self {
//             DspNode::Synth(ref mut phase) => dsp::slice::map_in_place(buffer, |_| {
//                 let val = sine_wave(*phase);
//                 const SYNTH_HZ: f64 = 110.0;
//                 *phase += SYNTH_HZ / sample_hz;
//                 Frame::from_fn(|_| val)
//             }),
//             DspNode::Volume(vol) => dsp::slice::map_in_place(buffer, |f| f.map(|s| Frame::mul_amp(s, vol))),
//         }
//     }
// }

// fn sine_wave<S: Sample>(phase: f64) -> S
// where
//     S: Sample + FromSample<f32>,
// {
//     use std::f64::consts::PI;
//     ((phase * PI * 2.0).sin() as f32).to_sample::<S>()
// }



fn sine_wave<S: Sample>(phase: Phase, volume: Volume) -> S
where
    S: Sample + FromSample<f32>,
{
    use std::f64::consts::PI;
    ((phase * PI * 2.0).sin() as f32 * volume).to_sample::<S>()
}
