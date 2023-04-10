use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AudioPlugin).add_startup_system(start_music);
    }
}

fn start_music(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio
        .play(asset_server.load("audio/PotionPanic.wav"))
        .looped();
}
