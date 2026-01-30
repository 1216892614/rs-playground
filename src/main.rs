use bevy::{asset::io::web::WebAssetPlugin, prelude::*};

mod canvas;
mod main_menu;
mod pixel_editor;
mod save_load;
mod status_bar;
mod theme;

use canvas::Canvas;

/// 应用页面状态
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    PixelEditor,
    /// 开始游戏：存档/章节选择时间轴
    SaveLoad,
}

/// 系统顺序：当前页面的 cell 事件先消费（避免 hover 被主菜单抢先消费），再主菜单/状态栏
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppSet {
    /// 存档页悬浮/点击，必须在 MainMenuCell 前运行否则 hover 会被 main_menu 消费掉
    SaveLoadCell,
    MainMenuCell,
    StatusBarCell,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WebAssetPlugin {
            silence_startup_warning: true,
        }))
        .init_state::<AppState>()
        .add_plugins(theme::ThemePlugin)
        .add_plugins(canvas::CanvasPlugin)
        .add_plugins(main_menu::MainMenuPlugin)
        .add_plugins(status_bar::StatusBarPlugin)
        .add_plugins(pixel_editor::PixelEditorPlugin)
        .add_plugins(save_load::SaveLoadPlugin)
        .configure_sets(
            Update,
            (
                AppSet::SaveLoadCell,
                AppSet::MainMenuCell,
                AppSet::StatusBarCell,
            )
                .chain(),
        )
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(OnEnter(AppState::MainMenu), clear_canvas_on_enter_main_menu)
        .add_systems(OnEnter(AppState::SaveLoad), clear_canvas_on_enter_save_load)
        .run();
}

// ==================== 启动设置 ====================

fn setup(mut commands: Commands, mut canvas: ResMut<Canvas>) {
    commands.spawn(Camera2d::default());
    canvas.clear();
    // 最下方两行由 StatusBarPlugin 绘制
}

fn clear_canvas_on_enter_main_menu(mut canvas: ResMut<Canvas>) {
    canvas.clear();
}

fn clear_canvas_on_enter_save_load(mut canvas: ResMut<Canvas>) {
    canvas.clear();
}
