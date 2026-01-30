use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

// ==================== Theme Plugin ====================

pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Theme::default())
            .add_systems(Startup, load_theme);
    }
}

// ==================== 配置结构 ====================

#[derive(Debug, Clone, Deserialize)]
struct ThemeConfig {
    theme: ThemeMeta,
    colors: ColorCategories,
}

#[derive(Debug, Clone, Deserialize)]
struct ThemeMeta {
    name: String,
    style: String,
    description: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ColorCategories {
    background: HashMap<String, String>,
    earth: HashMap<String, String>,
    text: HashMap<String, String>,
    semantic: HashMap<String, String>,
    rarity: HashMap<String, String>,
    grayscale: HashMap<String, String>,
}

// ==================== 主题资源 ====================

/// 主题资源 - 提供便捷的颜色访问
#[derive(Resource, Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub style: String,
    pub description: String,
    
    // 背景色
    pub bg: BackgroundColors,
    // 地形/世界色
    pub earth: EarthColors,
    // 文字色
    pub text: TextColors,
    // 语义色/UI状态
    pub semantic: SemanticColors,
    // 稀有度
    pub rarity: RarityColors,
    // 灰度梯度
    pub grayscale: GrayscaleColors,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            style: "default".to_string(),
            description: "Default theme".to_string(),
            bg: BackgroundColors::default(),
            earth: EarthColors::default(),
            text: TextColors::default(),
            semantic: SemanticColors::default(),
            rarity: RarityColors::default(),
            grayscale: GrayscaleColors::default(),
        }
    }
}

// ==================== 颜色分类结构 ====================

#[derive(Debug, Clone)]
pub struct BackgroundColors {
    pub void_ink: Color,
    pub basalt_blue: Color,
    pub obsidian_moss: Color,
    pub ashen_slate: Color,
}

impl Default for BackgroundColors {
    fn default() -> Self {
        Self {
            void_ink: Color::BLACK,
            basalt_blue: Color::srgb(0.1, 0.13, 0.19),
            obsidian_moss: Color::srgb(0.16, 0.18, 0.17),
            ashen_slate: Color::srgb(0.43, 0.42, 0.37),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EarthColors {
    pub ruined_umber: Color,
    pub desert_bronze: Color,
    pub silt_gold: Color,
    pub sunbaked_clay: Color,
    pub bone_parchment: Color,
}

impl Default for EarthColors {
    fn default() -> Self {
        Self {
            ruined_umber: Color::srgb(0.23, 0.17, 0.11),
            desert_bronze: Color::srgb(0.36, 0.27, 0.15),
            silt_gold: Color::srgb(0.55, 0.42, 0.23),
            sunbaked_clay: Color::srgb(0.76, 0.64, 0.43),
            bone_parchment: Color::srgb(0.95, 0.90, 0.78),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextColors {
    pub primary: Color,
    pub secondary: Color,
    pub muted: Color,
    pub disabled: Color,
}

impl Default for TextColors {
    fn default() -> Self {
        Self {
            primary: Color::WHITE,
            secondary: Color::srgb(0.66, 0.58, 0.48),
            muted: Color::srgb(0.43, 0.42, 0.37),
            disabled: Color::srgb(0.29, 0.31, 0.33),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SemanticColors {
    pub success: Color,
    pub info: Color,
    pub warning: Color,
    pub danger: Color,
    pub fire: Color,
    pub toxic: Color,
    pub psi: Color,
    pub rare: Color,
    pub tech_neon: Color,
}

impl Default for SemanticColors {
    fn default() -> Self {
        Self {
            success: Color::srgb(0.19, 0.82, 0.48),
            info: Color::srgb(0.16, 0.56, 0.63),
            warning: Color::srgb(0.90, 0.83, 0.29),
            danger: Color::srgb(0.88, 0.29, 0.24),
            fire: Color::srgb(1.0, 0.60, 0.24),
            toxic: Color::srgb(0.18, 0.44, 0.37),
            psi: Color::srgb(0.48, 0.31, 0.85),
            rare: Color::srgb(0.84, 0.30, 0.80),
            tech_neon: Color::srgb(0.30, 0.95, 1.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RarityColors {
    pub common: Color,
    pub uncommon: Color,
    pub rare: Color,
    pub epic: Color,
    pub legendary: Color,
    pub artifact: Color,
}

impl Default for RarityColors {
    fn default() -> Self {
        Self {
            common: Color::srgb(0.66, 0.58, 0.48),
            uncommon: Color::srgb(0.19, 0.82, 0.48),
            rare: Color::srgb(0.15, 0.39, 0.84),
            epic: Color::srgb(0.48, 0.31, 0.85),
            legendary: Color::srgb(0.84, 0.30, 0.80),
            artifact: Color::srgb(0.30, 0.95, 1.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GrayscaleColors {
    pub g00_black: Color,
    pub g05_charcoal: Color,
    pub g10_ink: Color,
    pub g15_dark: Color,
    pub g20_graphite: Color,
    pub g30_slate: Color,
    pub g40_ash: Color,
    pub g50_mid: Color,
    pub g60_fog: Color,
    pub g70_mist: Color,
    pub g80_silver: Color,
    pub g90_pale: Color,
    pub g100_white: Color,
}

impl Default for GrayscaleColors {
    fn default() -> Self {
        Self {
            g00_black: Color::srgb(0.0, 0.0, 0.0),
            g05_charcoal: Color::srgb(0.05, 0.05, 0.07),
            g10_ink: Color::srgb(0.08, 0.09, 0.11),
            g15_dark: Color::srgb(0.11, 0.13, 0.15),
            g20_graphite: Color::srgb(0.15, 0.16, 0.20),
            g30_slate: Color::srgb(0.20, 0.23, 0.27),
            g40_ash: Color::srgb(0.29, 0.31, 0.33),
            g50_mid: Color::srgb(0.42, 0.44, 0.47),
            g60_fog: Color::srgb(0.54, 0.56, 0.60),
            g70_mist: Color::srgb(0.66, 0.68, 0.71),
            g80_silver: Color::srgb(0.77, 0.79, 0.82),
            g90_pale: Color::srgb(0.88, 0.89, 0.91),
            g100_white: Color::srgb(1.0, 1.0, 1.0),
        }
    }
}

// ==================== 辅助函数 ====================

/// 从 hex 字符串解析颜色
fn parse_hex_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        warn!("Invalid hex color: {}", hex);
        return Color::WHITE;
    }
    
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255) as f32 / 255.0;
    
    Color::srgb(r, g, b)
}

// ==================== 主题加载系统 ====================

/// 加载主题系统
fn load_theme(mut theme: ResMut<Theme>, mut clear_color: ResMut<ClearColor>) {
    let theme_path = "themes/epic_desert_relic.toml";
    
    info!("Loading theme from: {}", theme_path);
    
    // 读取主题文件
    let theme_content = match std::fs::read_to_string(format!("assets/{}", theme_path)) {
        Ok(content) => content,
        Err(e) => {
            error!("Failed to read theme file: {}", e);
            warn!("Using default theme");
            return;
        }
    };
    
    // 解析 TOML
    let config: ThemeConfig = match toml::from_str(&theme_content) {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to parse theme file: {}", e);
            warn!("Using default theme");
            return;
        }
    };
    
    // 更新主题资源
    theme.name = config.theme.name;
    theme.style = config.theme.style;
    theme.description = config.theme.description;
    
    // 加载背景色
    if let Some(color) = config.colors.background.get("void_ink") {
        theme.bg.void_ink = parse_hex_color(color);
    }
    if let Some(color) = config.colors.background.get("basalt_blue") {
        theme.bg.basalt_blue = parse_hex_color(color);
    }
    if let Some(color) = config.colors.background.get("obsidian_moss") {
        theme.bg.obsidian_moss = parse_hex_color(color);
    }
    if let Some(color) = config.colors.background.get("ashen_slate") {
        theme.bg.ashen_slate = parse_hex_color(color);
    }
    
    // 加载地形色
    if let Some(color) = config.colors.earth.get("ruined_umber") {
        theme.earth.ruined_umber = parse_hex_color(color);
    }
    if let Some(color) = config.colors.earth.get("desert_bronze") {
        theme.earth.desert_bronze = parse_hex_color(color);
    }
    if let Some(color) = config.colors.earth.get("silt_gold") {
        theme.earth.silt_gold = parse_hex_color(color);
    }
    if let Some(color) = config.colors.earth.get("sunbaked_clay") {
        theme.earth.sunbaked_clay = parse_hex_color(color);
    }
    if let Some(color) = config.colors.earth.get("bone_parchment") {
        theme.earth.bone_parchment = parse_hex_color(color);
    }
    
    // 加载文字色
    if let Some(color) = config.colors.text.get("primary") {
        theme.text.primary = parse_hex_color(color);
    }
    if let Some(color) = config.colors.text.get("secondary") {
        theme.text.secondary = parse_hex_color(color);
    }
    if let Some(color) = config.colors.text.get("muted") {
        theme.text.muted = parse_hex_color(color);
    }
    if let Some(color) = config.colors.text.get("disabled") {
        theme.text.disabled = parse_hex_color(color);
    }
    
    // 加载语义色
    if let Some(color) = config.colors.semantic.get("success") {
        theme.semantic.success = parse_hex_color(color);
    }
    if let Some(color) = config.colors.semantic.get("info") {
        theme.semantic.info = parse_hex_color(color);
    }
    if let Some(color) = config.colors.semantic.get("warning") {
        theme.semantic.warning = parse_hex_color(color);
    }
    if let Some(color) = config.colors.semantic.get("danger") {
        theme.semantic.danger = parse_hex_color(color);
    }
    if let Some(color) = config.colors.semantic.get("fire") {
        theme.semantic.fire = parse_hex_color(color);
    }
    if let Some(color) = config.colors.semantic.get("toxic") {
        theme.semantic.toxic = parse_hex_color(color);
    }
    if let Some(color) = config.colors.semantic.get("psi") {
        theme.semantic.psi = parse_hex_color(color);
    }
    if let Some(color) = config.colors.semantic.get("rare") {
        theme.semantic.rare = parse_hex_color(color);
    }
    if let Some(color) = config.colors.semantic.get("tech_neon") {
        theme.semantic.tech_neon = parse_hex_color(color);
    }
    
    // 加载稀有度色
    if let Some(color) = config.colors.rarity.get("common") {
        theme.rarity.common = parse_hex_color(color);
    }
    if let Some(color) = config.colors.rarity.get("uncommon") {
        theme.rarity.uncommon = parse_hex_color(color);
    }
    if let Some(color) = config.colors.rarity.get("rare") {
        theme.rarity.rare = parse_hex_color(color);
    }
    if let Some(color) = config.colors.rarity.get("epic") {
        theme.rarity.epic = parse_hex_color(color);
    }
    if let Some(color) = config.colors.rarity.get("legendary") {
        theme.rarity.legendary = parse_hex_color(color);
    }
    if let Some(color) = config.colors.rarity.get("artifact") {
        theme.rarity.artifact = parse_hex_color(color);
    }
    
    // 加载灰度色
    if let Some(color) = config.colors.grayscale.get("g00_black") {
        theme.grayscale.g00_black = parse_hex_color(color);
    }
    if let Some(color) = config.colors.grayscale.get("g05_charcoal") {
        theme.grayscale.g05_charcoal = parse_hex_color(color);
    }
    if let Some(color) = config.colors.grayscale.get("g10_ink") {
        theme.grayscale.g10_ink = parse_hex_color(color);
    }
    if let Some(color) = config.colors.grayscale.get("g15_dark") {
        theme.grayscale.g15_dark = parse_hex_color(color);
    }
    if let Some(color) = config.colors.grayscale.get("g20_graphite") {
        theme.grayscale.g20_graphite = parse_hex_color(color);
    }
    if let Some(color) = config.colors.grayscale.get("g30_slate") {
        theme.grayscale.g30_slate = parse_hex_color(color);
    }
    if let Some(color) = config.colors.grayscale.get("g40_ash") {
        theme.grayscale.g40_ash = parse_hex_color(color);
    }
    if let Some(color) = config.colors.grayscale.get("g50_mid") {
        theme.grayscale.g50_mid = parse_hex_color(color);
    }
    if let Some(color) = config.colors.grayscale.get("g60_fog") {
        theme.grayscale.g60_fog = parse_hex_color(color);
    }
    if let Some(color) = config.colors.grayscale.get("g70_mist") {
        theme.grayscale.g70_mist = parse_hex_color(color);
    }
    if let Some(color) = config.colors.grayscale.get("g80_silver") {
        theme.grayscale.g80_silver = parse_hex_color(color);
    }
    if let Some(color) = config.colors.grayscale.get("g90_pale") {
        theme.grayscale.g90_pale = parse_hex_color(color);
    }
    if let Some(color) = config.colors.grayscale.get("g100_white") {
        theme.grayscale.g100_white = parse_hex_color(color);
    }
    
    // 更新背景颜色为主题背景色
    clear_color.0 = theme.bg.void_ink;
    
    info!("✓ Theme loaded: {} ({})", theme.name, theme.style);
    info!("  {}", theme.description);
}
