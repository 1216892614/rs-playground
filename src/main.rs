use bevy::{asset::io::web::WebAssetPlugin, prelude::*};

mod canvas;
mod theme;

use canvas::{Canvas, TextAlign};
use theme::Theme;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WebAssetPlugin {
            silence_startup_warning: true,
        }))
        .add_plugins(theme::ThemePlugin)
        .add_plugins(canvas::CanvasPlugin)
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, demo_setup)
        .run();
}

// ==================== 演示设置 ====================

fn demo_setup(mut commands: Commands, mut canvas: ResMut<Canvas>, theme: Res<Theme>) {
    // 创建相机
    commands.spawn(Camera2d::default());

    // 绘制主题演示（使用新 API：y, x_start, x_end, text, align, color）
    canvas.set_line(
        2,
        2,
        50,
        "Epic Desert Relic 主题演示",
        TextAlign::Left,
        theme.text.primary,
    );
    canvas.set_line(
        4,
        2,
        50,
        "96x54 Grid Canvas",
        TextAlign::Left,
        theme.text.secondary,
    );

    // 绘制边框 - 使用主题边框色
    for x in 0..96 {
        canvas.set_char(x, 0, '─', theme.bg.ashen_slate);
        canvas.set_char(x, 53, '─', theme.bg.ashen_slate);
    }
    for y in 0..54 {
        canvas.set_char(0, y, '│', theme.bg.ashen_slate);
        canvas.set_char(95, y, '│', theme.bg.ashen_slate);
    }
    // 四个角
    canvas.set_char(0, 0, '┌', theme.bg.ashen_slate);
    canvas.set_char(95, 0, '┐', theme.bg.ashen_slate);
    canvas.set_char(0, 53, '└', theme.bg.ashen_slate);
    canvas.set_char(95, 53, '┘', theme.bg.ashen_slate);

    // 地形色演示
    canvas.set_line(
        7,
        2,
        22,
        "═══ 地形色 ═══",
        TextAlign::Left,
        theme.earth.silt_gold,
    );
    canvas.set_char(2, 9, '█', theme.earth.ruined_umber);
    canvas.set_line(9, 4, 20, "岩壁/皮革", TextAlign::Left, theme.text.muted);
    canvas.set_char(2, 10, '█', theme.earth.desert_bronze);
    canvas.set_line(10, 4, 20, "沙砾/金属", TextAlign::Left, theme.text.muted);
    canvas.set_char(2, 11, '█', theme.earth.silt_gold);
    canvas.set_line(11, 4, 20, "沙丘高光", TextAlign::Left, theme.text.muted);
    canvas.set_char(2, 12, '█', theme.earth.sunbaked_clay);
    canvas.set_line(12, 4, 20, "浅沙/石板", TextAlign::Left, theme.text.muted);
    canvas.set_char(2, 13, '█', theme.earth.bone_parchment);
    canvas.set_line(13, 4, 20, "亮地表/骨质", TextAlign::Left, theme.text.muted);

    // 语义色演示
    canvas.set_line(
        7,
        25,
        45,
        "═══ 语义色 ═══",
        TextAlign::Left,
        theme.semantic.tech_neon,
    );
    canvas.set_line(9, 25, 34, "✓ 成功", TextAlign::Left, theme.semantic.success);
    canvas.set_line(10, 25, 34, "ℹ 信息", TextAlign::Left, theme.semantic.info);
    canvas.set_line(
        11,
        25,
        34,
        "⚠ 警告",
        TextAlign::Left,
        theme.semantic.warning,
    );
    canvas.set_line(12, 25, 34, "✖ 危险", TextAlign::Left, theme.semantic.danger);
    canvas.set_line(13, 25, 34, " 火焰", TextAlign::Left, theme.semantic.fire);
    canvas.set_line(9, 34, 45, " 毒素", TextAlign::Left, theme.semantic.toxic);
    canvas.set_line(10, 34, 45, "◈ 灵能", TextAlign::Left, theme.semantic.psi);
    canvas.set_line(11, 34, 45, "★ 稀有", TextAlign::Left, theme.semantic.rare);
    canvas.set_line(
        12,
        34,
        45,
        "◆ 高科技",
        TextAlign::Left,
        theme.semantic.tech_neon,
    );

    // 稀有度演示
    canvas.set_line(
        7,
        50,
        70,
        "═══ 稀有度 ═══",
        TextAlign::Left,
        theme.earth.bone_parchment,
    );
    canvas.fill_rect_with_bg(50, 9, 20, 1, ' ', theme.rarity.common, theme.bg.basalt_blue);
    canvas.set_line(9, 51, 71, "普通", TextAlign::Left, theme.rarity.common);
    canvas.fill_rect_with_bg(
        50,
        10,
        20,
        1,
        ' ',
        theme.rarity.uncommon,
        theme.bg.basalt_blue,
    );
    canvas.set_line(10, 51, 71, "罕见", TextAlign::Left, theme.rarity.uncommon);
    canvas.fill_rect_with_bg(50, 11, 20, 1, ' ', theme.rarity.rare, theme.bg.basalt_blue);
    canvas.set_line(11, 51, 71, "稀有", TextAlign::Left, theme.rarity.rare);
    canvas.fill_rect_with_bg(50, 12, 20, 1, ' ', theme.rarity.epic, theme.bg.basalt_blue);
    canvas.set_line(12, 51, 71, "史诗", TextAlign::Left, theme.rarity.epic);
    canvas.fill_rect_with_bg(
        50,
        13,
        20,
        1,
        ' ',
        theme.rarity.legendary,
        theme.bg.basalt_blue,
    );
    canvas.set_line(13, 51, 71, "传说", TextAlign::Left, theme.rarity.legendary);
    canvas.fill_rect_with_bg(
        50,
        14,
        20,
        1,
        ' ',
        theme.rarity.artifact,
        theme.bg.basalt_blue,
    );
    canvas.set_line(14, 51, 71, "神器", TextAlign::Left, theme.rarity.artifact);

    // 灰度梯度演示
    canvas.set_line(
        16,
        2,
        40,
        "═══ 灰度梯度 (FOV/雾/距离) ═══",
        TextAlign::Left,
        theme.text.secondary,
    );
    for i in 0..13 {
        let color = match i {
            0 => theme.grayscale.g00_black,
            1 => theme.grayscale.g05_charcoal,
            2 => theme.grayscale.g10_ink,
            3 => theme.grayscale.g15_dark,
            4 => theme.grayscale.g20_graphite,
            5 => theme.grayscale.g30_slate,
            6 => theme.grayscale.g40_ash,
            7 => theme.grayscale.g50_mid,
            8 => theme.grayscale.g60_fog,
            9 => theme.grayscale.g70_mist,
            10 => theme.grayscale.g80_silver,
            11 => theme.grayscale.g90_pale,
            12 => theme.grayscale.g100_white,
            _ => Color::WHITE,
        };
        canvas.set_char(2 + i * 2, 18, '█', color);
    }

    // 背景色演示
    canvas.set_line(
        21,
        2,
        25,
        "═══ 背景/面板 ═══",
        TextAlign::Left,
        theme.text.primary,
    );
    canvas.fill_rect_with_bg(2, 23, 15, 3, ' ', theme.text.primary, theme.bg.void_ink);
    canvas.set_line(24, 3, 18, "主背景", TextAlign::Center, theme.text.primary);

    canvas.fill_rect_with_bg(18, 23, 15, 3, ' ', theme.text.primary, theme.bg.basalt_blue);
    canvas.set_line(24, 19, 33, "面板底", TextAlign::Center, theme.text.primary);

    canvas.fill_rect_with_bg(
        34,
        23,
        15,
        3,
        ' ',
        theme.text.primary,
        theme.bg.obsidian_moss,
    );
    canvas.set_line(
        24,
        35,
        49,
        "中性暗绿",
        TextAlign::Center,
        theme.text.primary,
    );

    // 文字层次演示
    canvas.set_line(
        28,
        2,
        22,
        "═══ 文字层次 ═══",
        TextAlign::Left,
        theme.text.primary,
    );
    canvas.set_line(30, 2, 20, "主要文字", TextAlign::Left, theme.text.primary);
    canvas.set_line(31, 2, 20, "次要文字", TextAlign::Left, theme.text.secondary);
    canvas.set_line(32, 2, 20, "静默文字", TextAlign::Left, theme.text.muted);
    canvas.set_line(33, 2, 20, "禁用文字", TextAlign::Left, theme.text.disabled);

    // 实战场景示例
    canvas.set_line(
        36,
        2,
        45,
        "═══ 实战场景示例 ═══",
        TextAlign::Left,
        theme.semantic.tech_neon,
    );

    // UI 面板
    canvas.fill_rect_with_bg(2, 38, 40, 8, ' ', theme.text.primary, theme.bg.basalt_blue);
    canvas.fill_rect_with_bg(
        3,
        39,
        38,
        1,
        ' ',
        theme.text.primary,
        theme.bg.obsidian_moss,
    );
    canvas.set_line(39, 4, 20, "物品栏", TextAlign::Left, theme.text.primary);

    // 物品列表
    canvas.set_line(
        41,
        4,
        45,
        "[普通] 生锈的剑",
        TextAlign::Left,
        theme.rarity.common,
    );
    canvas.set_line(
        42,
        4,
        45,
        "[稀有] 沙漠护符",
        TextAlign::Left,
        theme.rarity.rare,
    );
    canvas.set_line(
        43,
        4,
        45,
        "[神器] 古代遗物",
        TextAlign::Left,
        theme.rarity.artifact,
    );

    // 状态提示
    canvas.set_line(41, 25, 35, "HP:", TextAlign::Left, theme.text.secondary);
    canvas.set_line(
        41,
        29,
        45,
        "85/100",
        TextAlign::Left,
        theme.semantic.success,
    );
    canvas.set_line(42, 25, 35, "MP:", TextAlign::Left, theme.text.secondary);
    canvas.set_line(42, 29, 45, "42/80", TextAlign::Left, theme.semantic.info);
    canvas.set_line(43, 25, 35, "状态:", TextAlign::Left, theme.text.secondary);
    canvas.set_line(43, 30, 45, "中毒", TextAlign::Left, theme.semantic.toxic);

    // 按钮
    canvas.fill_rect_with_bg(4, 44, 10, 1, ' ', theme.text.primary, theme.bg.ashen_slate);
    canvas.set_line(44, 5, 15, "[ 使用 ]", TextAlign::Center, theme.text.primary);

    // 图标测试
    canvas.set_line(
        36,
        50,
        65,
        " 图标测试:",
        TextAlign::Left,
        theme.text.secondary,
    );
    canvas.set_line(38, 50, 55, "   ", TextAlign::Left, theme.semantic.warning);
    canvas.set_line(39, 50, 55, "   ", TextAlign::Left, theme.semantic.info);
    canvas.set_line(40, 50, 55, "   ", TextAlign::Left, theme.semantic.danger);
}
