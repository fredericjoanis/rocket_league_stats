use boxcars::Replay;
use boxcars::RigidBody;
use boxcars::{ActorId, ObjectId};

use ahash::RandomState;
use std::collections::HashMap;
use std::error::{self};
use std::fs::{self, File};
use std::time::Instant;

fn main() -> Result<(), Box<dyn error::Error>> {
    let filename = r"data\EBF5B75F4A6B60FF1F612AA134135E69.replay";

    let buffer = fs::read(filename)?;
    let replay = boxcars::ParserBuilder::new(&buffer)
        .must_parse_network_data()
        .parse()?;

    let start = Instant::now();
    map_player_positions(&replay)?;
    //save_as_json(&replay)?;

    println!("Completed in {:?}", start.elapsed());

    Ok(())
}

fn map_player_positions(replay: &Replay) -> Result<(), Box<dyn error::Error>> {
    let object_id_player = find_object_id(replay, "Engine.Pawn:PlayerReplicationInfo")?;
    let object_id_name = find_object_id(replay, "Engine.PlayerReplicationInfo:PlayerName")?;
    let car_calculus = find_object_id(replay, "TAGame.RBActor_TA:ReplicatedRBState")?;

    let network = replay.network_frames.as_ref().unwrap();

    let mut positions: HashMap<ActorId, Vec<CarCalculus>, RandomState> = HashMap::default();
    let mut actor_id_to_player: HashMap<ActorId, ActorId, RandomState> = HashMap::default();
    let mut actor_id_name: HashMap<ActorId, String, RandomState> = HashMap::default();

    for frame in &network.frames {
        for updated_attribute in &frame.updated_actors {
            // We have a new mapping for actor_id to player
            if updated_attribute.object_id == object_id_player {
                if let boxcars::Attribute::ActiveActor(player) = updated_attribute.attribute {
                    if player.active {
                        actor_id_to_player.insert(updated_attribute.actor_id, player.actor);
                    }
                }
            } else if updated_attribute.object_id == car_calculus {
                // New Position, Velocity and acceleration
                if let boxcars::Attribute::RigidBody(car) = updated_attribute.attribute {
                    if let Some(player_actor_id) =
                        actor_id_to_player.get(&updated_attribute.actor_id)
                    {
                        let entry = positions.entry(*player_actor_id).or_insert_with(Vec::new);

                        entry.push(CarCalculus::new(frame.time, car));
                    }
                }
            } else if updated_attribute.object_id == object_id_name {
                if let boxcars::Attribute::String(player_name) = &updated_attribute.attribute {
                    actor_id_name.insert(updated_attribute.actor_id, player_name.clone());
                }
            }
        }
    }

    for (actor, car) in positions {
        println!(
            "{} length {}",
            actor_id_name.get(&actor).unwrap_or(&actor.to_string()),
            car.len()
        );
    }

    Ok(())
}

#[allow(dead_code)]
fn save_as_json(replay: &Replay) -> Result<(), Box<dyn error::Error>> {
    serde_json::to_writer_pretty(&File::create(r#"E:\replay.json"#)?, &replay)?;
    Ok(())
}

#[allow(dead_code)]
struct CarCalculus {
    frame_time: f32,
    rigid_body: RigidBody,
}

impl CarCalculus {
    fn new(frame_time: f32, rigid_body: RigidBody) -> Self {
        CarCalculus {
            frame_time,
            rigid_body,
        }
    }
}

fn find_object_id(replay: &Replay, name: &str) -> Result<ObjectId, Box<dyn error::Error>> {
    let id = replay
        .objects
        .iter()
        .position(|val| val == name)
        .map(|index| boxcars::ObjectId(index as i32))
        .ok_or_else(|| format!("Expected {} to be present in replay", name))?;
    Ok(id)
}
