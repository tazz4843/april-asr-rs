#![feature(slice_as_chunks)]

use april_asr_rs::*;
use std::sync::atomic::AtomicU64;

struct Data {
    transcription_id: u32,
    times_called: AtomicU64,
}

fn main() {
    let model = AprilModel::new("/home/niko/data/april-asr/models/aprilv0_en-us.april")
        .expect("failed to load model");
    let model_name = model.get_model_name().expect("failed to get model name");
    let model_description = model
        .get_model_description()
        .expect("failed to get model description");
    let model_language = model
        .get_model_language()
        .expect("failed to get model language");
    let model_sample_rate = model.get_sample_rate();
    println!(
        "Running {} ({}), language {} at an expected sample rate of {}Hz",
        model_name, model_description, model_language, model_sample_rate
    );

    let mut config = AprilConfig::<Data>::default();
    let internal_data = Data {
        transcription_id: 0,
        times_called: AtomicU64::new(0),
    };
    config.set_handler_fn(april_callback, internal_data);
    let mut session = model
        .create_session(config)
        .expect("failed to start session");

    let raw_data = include_bytes!("../jfk.raw");
    let mut samples = vec![0; raw_data.len() / 2];
    for sample in raw_data.as_chunks::<2>().0 {
        samples.push(i16::from_le_bytes(*sample));
    }
    session.feed_pcm16(&mut samples[..]);
    session.flush();
}

fn april_callback(data: &Data, result: AprilResultType, tokens: AprilTokens) {
    data.times_called
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let called_times = data.times_called.load(std::sync::atomic::Ordering::SeqCst);
    println!(
        "transcription ID {}: called {} times, new result: {}, with tokens {}",
        data.transcription_id, called_times, result, tokens
    );
}
