use crate::AppState;
use axum::extract::State;
use bevy_ecs::{component::Component, entity::Entity};
use color_eyre::eyre::ContextCompat;
use log::warn;
use trc_protocol::computer_packets::ComputerToServerPacket;


#[derive(Clone, PartialEq, Eq, Hash, Component)]
pub struct ComputerIdent {
    pub id: i32,
    pub world: String,
}

pub async fn handle_computer_packet(
    packet: ComputerToServerPacket,
    state: &State<AppState>,
    entity: Entity,
) -> color_eyre::Result<()> {
    match packet {
        ComputerToServerPacket::StdOut(std_out) => {
            warn!("recived unimplemented package: StdOut: {}", std_out);
        }
        ComputerToServerPacket::Executables(executables) => {
            warn!(
                "recived unimplemented package: Executables: {:#?}",
                executables
            );
        }
        ComputerToServerPacket::UpdateName(name) => {
            let world = state.read().await;
            let ident = world
                .get::<ComputerIdent>(entity)
                .context("can't get ident for computer")?;
            sqlx::query!(
                "UPDATE turtles SET name = ? WHERE id = ? AND world = ?;",
                name,
                ident.id,
                ident.world
            )
            .execute(&state.db)
            .await
            .unwrap();
        }
        ComputerToServerPacket::ChangeWorld(new_world) => {
            let ecs_world = state.read().await;
            let ident = ecs_world
                .get::<ComputerIdent>(entity)
                .context("can't get ident for computer")?;
            sqlx::query!(
                "UPDATE turtles SET world = ? WHERE id = ? AND world = ?;",
                new_world,
                ident.id,
                ident.world
            )
            .execute(&state.db)
            .await
            .unwrap();
            drop(ecs_world);
            state
                .write()
                .await
                .get_mut::<ComputerIdent>(entity)
                .context("can't get ident for computer")?
                .world = new_world;
        }
        ComputerToServerPacket::Ping => todo!(),
    };

    Ok(())
}
