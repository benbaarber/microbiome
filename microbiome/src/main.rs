use std::{
    env,
    error::Error,
    thread,
    time::{Duration, Instant},
};

use microbiome::{invariants::FRAME_DURATION, Microbiome};

fn main() -> Result<(), Box<dyn Error>> {
    let mut mb = Microbiome::new();

    let pub_to = env::var("MB_PUBSUB").expect("MB_PUBSUB must be set");
    let context = zmq::Context::new();
    let pub_sock = context.socket(zmq::PUB)?;
    pub_sock.bind(&pub_to)?;

    let frame_duration = Duration::from_millis(FRAME_DURATION as u64);

    loop {
        let start = Instant::now();

        mb.step();

        let state = serde_json::to_vec(&mb)?;
        pub_sock.send_multipart(
            ["mb_state".as_bytes(), "state".as_bytes(), &state],
            zmq::DONTWAIT,
        )?;

        if let Some(sleep_duration) = frame_duration.checked_sub(start.elapsed()) {
            thread::sleep(sleep_duration);
        }
    }

    Ok(())
}
