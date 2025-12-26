use crate::state::gacha::parse_dupe_rewards;
use data::exceldb;

pub async fn grant_dupe_rewards(
    hero_id: i32,
    duplicate_count: i32,
) -> anyhow::Result<(Vec<(u32, i32)>, Vec<(i32, i32)>)> {
    if duplicate_count == 0 {
        return Ok((Vec::new(), Vec::new()));
    }

    let game_data = exceldb::get();
    let hero = game_data
        .character
        .get(hero_id)
        .ok_or_else(|| anyhow::anyhow!("Hero {} not found", hero_id))?;

    let (items, currencies) = if duplicate_count > 5 {
        parse_dupe_rewards(&hero.duplicate_item2)
    } else {
        parse_dupe_rewards(&hero.duplicate_item)
    };

    Ok((items, currencies))
}
