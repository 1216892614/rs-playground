//! 存档结构：每个 fork 一个存档文件，使用 rmp-serde 序列化。
//! 结构变更时 bump SAVE_VERSION，无法迁移兼容的旧存档会被自动抛弃并禁用开始游戏。
//! 版本比较使用 semver 语义化版本。

use rand::RngCore;
use semver::Version;
use serde::{Deserialize, Serialize};

// ==================== 版本与章节常量 ====================

/// 存档格式语义化版本；结构变更时修改此常量，旧版本存档将被视为不兼容并抛弃
pub const SAVE_VERSION: &str = "1.0.0";

/// 第一章的章节编号（const，用于 fork_from）
pub const CHAPTER_NO_FIRST: u32 = 1;

// ==================== 序列化用结构 ====================

/// Fork 来源：父存档 id + 章节编号（章节编号为游戏内 const）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForkFrom {
    pub parent_id: String,
    pub chapter_no: u32,
}

/// 时间轴上的一个槽位（与 TimelineNode 对应，用于序列化）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimelineSlot {
    pub is_chapter: bool,
    pub chapter_name: String,
    pub duration_display: String,
}

/// 单个 fork 的存档文件结构（每个 fork 一个文件，rmp-serde 序列化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSave {
    /// 存档唯一 id（cuid2）
    pub id: String,
    /// 显示名：新游戏为 "[00:00:00] 第一章"，fork 一般为「章节名 + 游戏时间」
    pub name: String,
    /// 存档格式版本，须与 SAVE_VERSION 一致否则视为不兼容
    pub version: String,
    /// 若为 fork，则记录来源存档 id 与章节编号
    pub fork_from: Option<ForkFrom>,
    /// 256-bit 种子，用于后续随机事件
    pub seed: [u8; 32],
    /// 该分支的时间轴槽位列表
    pub timeline: Vec<TimelineSlot>,
}

impl GameSave {
    /// 是否与当前 SAVE_VERSION 兼容（semver 比较）；不兼容的存档应抛弃并禁用开始游戏
    pub fn is_compatible(&self) -> bool {
        match (Version::parse(SAVE_VERSION), Version::parse(&self.version)) {
            (Ok(current), Ok(save)) => current == save,
            _ => false,
        }
    }

    /// 创建新游戏存档：id=cuid2，name="[00:00:00] 第一章"，无 fork_from，随机 seed，仅第一章一个槽位
    pub fn new_game() -> Self {
        let mut seed = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut seed);
        Self {
            id: cuid2::create_id(),
            name: "[00:00:00] 第一章".to_string(),
            version: SAVE_VERSION.to_string(),
            fork_from: None,
            seed,
            timeline: vec![TimelineSlot {
                is_chapter: true,
                chapter_name: "第一章".to_string(),
                duration_display: "00:00:00".to_string(),
            }],
        }
    }
}
