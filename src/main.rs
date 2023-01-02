use boxcars::{ParseError, Replay};
use std::error;
use std::fs;

fn main() {
    run(
        r#"C:\Users\Frédéric\Documents\My Games\Rocket League\TAGame\Demos\23B70ECE44DB1B3D32FF7EB2FAA1318D.replay"#,
    ).unwrap();
}

fn parse_rl(data: &[u8]) -> Result<Replay, ParseError> {
    boxcars::ParserBuilder::new(data)
        .must_parse_network_data()
        .parse()
}

fn run(filename: &str) -> Result<(), Box<dyn error::Error>> {
    let buffer = fs::read(filename)?;
    let replay = parse_rl(&buffer)?;

    if let Some(network_frames) = replay.network_frames {
        let frame = network_frames.frames[132].clone();
        if let boxcars::Attribute::RigidBody(rigid_body) = frame.updated_actors[2].attribute {
            println!("{:?}", rigid_body.linear_velocity.unwrap());
        }
    }

    // serde_json::to_writer(&File::create(r#"D:\replay.json"#)?, &replay)?;
    Ok(())
}
