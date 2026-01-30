use bevy::{asset::io::web::WebAssetPlugin, prelude::*};

mod canvas;
mod status_bar;
mod theme;

use canvas::Canvas;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WebAssetPlugin {
            silence_startup_warning: true,
        }))
        .add_plugins(theme::ThemePlugin)
        .add_plugins(canvas::CanvasPlugin)
        .add_plugins(status_bar::StatusBarPlugin)
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .run();
}

// ==================== 启动设置 ====================

fn setup(mut commands: Commands, mut canvas: ResMut<Canvas>) {
    commands.spawn(Camera2d::default());
    canvas.clear();
    // 最下方两行由 StatusBarPlugin 绘制
}
