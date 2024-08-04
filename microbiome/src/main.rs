use std::{env, error::Error, thread, time};

use microbiome::Microbiome;

fn main() -> Result<(), Box<dyn Error>> {
    let mb = Microbiome::new();

    let pub_to = env::var("MB_PUBSUB")?;
    let context = zmq::Context::new();
    let pub_sock = context.socket(zmq::PUB)?;
    pub_sock.bind(&pub_to)?;

    loop {
        thread::sleep(time::Duration::from_secs(5));
        println!("sending...");
        let state = serde_json::to_vec(&mb)?;
        pub_sock.send_multipart(
            ["mb_state".as_bytes(), "state".as_bytes(), &state],
            zmq::DONTWAIT,
        )?;
    }

    Ok(())
}
