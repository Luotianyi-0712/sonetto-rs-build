use sonettobuf;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct UserWeekwalkInfo {
    pub user_id: i64,
    pub time: i32,
    pub end_time: i32,
    pub max_layer: i32,
    pub issue_id: i32,
    pub is_pop_deep_rule: bool,
    pub is_open_deep: bool,
    pub is_pop_shallow_settle: bool,
    pub is_pop_deep_settle: bool,
    pub deep_progress: String,
    pub time_this_week: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct WeekwalkMap {
    pub id: i64,
    pub user_id: i64,
    pub map_id: i32,
    pub scene_id: i32,
    pub is_finish: i32,
    pub is_finished: i32,
    pub buff_id: i32,
    pub is_show_buff: bool,
    pub is_show_finished: bool,
    pub is_show_select_cd: bool,
}

#[derive(Debug, Clone, FromRow)]
pub struct WeekwalkBattle {
    pub battle_id: i32,
    pub star: i32,
    pub max_star: i32,
    pub hero_group_select: i32,
    pub element_id: i32,
}

impl From<WeekwalkBattle> for sonettobuf::BattleInfo {
    fn from(b: WeekwalkBattle) -> Self {
        sonettobuf::BattleInfo {
            battle_id: Some(b.battle_id),
            star: Some(b.star),
            max_star: Some(b.max_star),
            hero_ids: vec![], // Will be populated separately
            hero_group_select: Some(b.hero_group_select),
            element_id: Some(b.element_id),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WeekwalkElementInfo {
    pub element_id: i32,
    pub is_finish: bool,
    pub index_num: i32,
    pub history_list: Vec<String>,
    pub visible: bool,
}

impl From<WeekwalkElementInfo> for sonettobuf::WeekwalkElementInfo {
    fn from(e: WeekwalkElementInfo) -> Self {
        sonettobuf::WeekwalkElementInfo {
            element_id: Some(e.element_id),
            is_finish: Some(e.is_finish),
            index: Some(e.index_num),
            historylist: e.history_list,
            visible: Some(e.visible),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WeekwalkHeroInfo {
    pub hero_id: i32,
    pub cd: i32,
}

impl From<WeekwalkHeroInfo> for sonettobuf::WeekwalkHeroInfo {
    fn from(h: WeekwalkHeroInfo) -> Self {
        sonettobuf::WeekwalkHeroInfo {
            hero_id: Some(h.hero_id),
            cd: Some(h.cd),
        }
    }
}
