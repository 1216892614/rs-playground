//! 页面路由：路由枚举（可带数据）、路由栈、前进/返回导航；与 AppState 同步，不设子路由，平铺。

use bevy::prelude::*;

use crate::AppState;

// ==================== 路由数据（跳转时传给目标页） ====================

/// 主菜单路由数据（暂无字段，预留）
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MainMenuRouteData {}

/// 存档页路由数据（预留：如指定打开的分支/节点）
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SaveLoadRouteData {}

/// 像素编辑器路由数据（预留：如打开的文件）
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PixelEditorRouteData {}

/// 新建人物路由数据（预留：如从哪个存档的 id 开始创建人物）
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CharacterCreationRouteData {}

// ==================== 路由枚举 ====================

/// 页面路由（平铺，无子路由）；可带数据用于页面间通讯。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    MainMenu(Option<MainMenuRouteData>),
    SaveLoad(Option<SaveLoadRouteData>),
    PixelEditor(Option<PixelEditorRouteData>),
    CharacterCreation(Option<CharacterCreationRouteData>),
}

impl Route {
    pub fn main_menu() -> Self {
        Route::MainMenu(None)
    }
    pub fn main_menu_with(data: MainMenuRouteData) -> Self {
        Route::MainMenu(Some(data))
    }
    pub fn save_load() -> Self {
        Route::SaveLoad(None)
    }
    pub fn save_load_with(data: SaveLoadRouteData) -> Self {
        Route::SaveLoad(Some(data))
    }
    pub fn pixel_editor() -> Self {
        Route::PixelEditor(None)
    }
    pub fn pixel_editor_with(data: PixelEditorRouteData) -> Self {
        Route::PixelEditor(Some(data))
    }
    pub fn character_creation() -> Self {
        Route::CharacterCreation(None)
    }
    pub fn character_creation_with(data: CharacterCreationRouteData) -> Self {
        Route::CharacterCreation(Some(data))
    }
}

/// 路由 → AppState（用于 run_if(in_state(...))）
pub fn route_to_app_state(route: &Route) -> AppState {
    match route {
        Route::MainMenu(_) => AppState::MainMenu,
        Route::SaveLoad(_) => AppState::SaveLoad,
        Route::PixelEditor(_) => AppState::PixelEditor,
        Route::CharacterCreation(_) => AppState::CharacterCreation,
    }
}

// ==================== 导航事件 ====================

/// 进入下一级页面（压栈；上一页仅清界面，实例保留，返回可恢复）
#[derive(Event)]
pub struct NavigatePush(pub Route);

/// 返回上一页（出栈）
#[derive(Event)]
pub struct NavigatePop;

impl bevy::prelude::Message for NavigatePush {}
impl bevy::prelude::Message for NavigatePop {}

// ==================== 路由栈 ====================

/// 路由栈；栈顶为当前页，返回时出栈回到上一页。
#[derive(Resource, Default)]
pub struct RouteStack {
    pub stack: Vec<Route>,
}

impl RouteStack {
    pub fn current(&self) -> Option<&Route> {
        self.stack.last()
    }
    pub fn current_app_state(&self) -> Option<AppState> {
        self.current().map(route_to_app_state)
    }
    pub fn can_pop(&self) -> bool {
        self.stack.len() > 1
    }
}

// ==================== 插件与系统 ====================

pub struct RouterPlugin;

fn router_handle_navigation(
    mut stack: ResMut<RouteStack>,
    mut ev_push: MessageReader<NavigatePush>,
    mut ev_pop: MessageReader<NavigatePop>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let mut changed = false;
    for ev in ev_push.read() {
        stack.stack.push(ev.0.clone());
        changed = true;
    }
    for _ in ev_pop.read() {
        if stack.stack.len() > 1 {
            stack.stack.pop();
            changed = true;
        }
    }
    if changed {
        if let Some(app) = stack.current_app_state() {
            next_state.set(app);
        }
    }
}

fn router_init_stack(mut stack: ResMut<RouteStack>) {
    if stack.stack.is_empty() {
        stack.stack.push(Route::main_menu());
    }
}

impl Plugin for RouterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RouteStack>()
            .add_message::<NavigatePush>()
            .add_message::<NavigatePop>()
            .add_systems(Startup, router_init_stack)
            .add_systems(Update, router_handle_navigation);
    }
}
