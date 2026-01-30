use bevy::{asset::io::web::WebAssetPlugin, prelude::*};

mod canvas;
mod main_menu;
mod status_bar;
mod theme;

use canvas::Canvas;

/// 系统顺序：主菜单先写外部悬浮文本，状态栏再读，保证同帧生效
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppSet {
    MainMenuCell,
    StatusBarCell,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WebAssetPlugin {
            silence_startup_warning: true,
        }))
        .add_plugins(theme::ThemePlugin)
        .add_plugins(canvas::CanvasPlugin)
        .add_plugins(main_menu::MainMenuPlugin)
        .add_plugins(status_bar::StatusBarPlugin)
        .configure_sets(
            Update,
            (AppSet::MainMenuCell, AppSet::StatusBarCell).chain(),
        )
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
