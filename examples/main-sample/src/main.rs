#![feature(slice_as_chunks)]

use april_asr_rs::*;
use std::sync::{Arc, Barrier};

fn main() {
    let model = AprilModel::new("/home/niko/data/april-asr/models/aprilv0_en-us.april")
        .expect("failed to load model");

    let mut config = AprilConfig::default();
    let barrier = Arc::new(Barrier::new(2));

    let barrier2 = Arc::clone(&barrier);
    config.set_handler_fn(Some(Box::new(move |x, y| {
        april_callback(x, y, barrier2.clone())
    })));
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
    barrier.wait();
}

fn april_callback(result: AprilResultType, tokens: AprilTokens, barrier: Arc<Barrier>) {
    println!("result: {}", result);
    println!("tokens: {}", tokens);

    if result == AprilResultType::RecognitionFinal {
        barrier.wait();
    }
}
