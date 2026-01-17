use sonettobuf;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Achievement {
    pub user_id: i64,
    pub achievement_id: i32,
    pub progress: i32,
    pub has_finish: bool,
    pub is_new: bool,
    pub finish_time: i32,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<Achievement> for sonettobuf::AchievementTaskInfo {
    fn from(a: Achievement) -> Self {
        sonettobuf::AchievementTaskInfo {
            id: Some(a.achievement_id),
            progress: Some(a.progress),
            has_finish: Some(a.has_finish),
            new: Some(a.is_new),
            finish_time: Some(a.finish_time),
        }
    }
}
