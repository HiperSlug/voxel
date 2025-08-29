use bevy::{prelude::*, render::{Render, RenderApp, RenderSet}};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, OnRenderStartup))
        .run();
}

struct OnRenderStartup;

impl Plugin for OnRenderStartup {
    fn build(&self, app: &mut App) {
        app
            .sub_app_mut(RenderApp)
            .add_systems(Render, info);
    }
}

fn info() {
    info!("_");
}
