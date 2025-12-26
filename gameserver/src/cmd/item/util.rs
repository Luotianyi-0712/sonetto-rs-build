use rand::{seq::SliceRandom, thread_rng};

use crate::state::{get_rewards, parse_item};

pub fn process_item_use(
    material_id: u32,
    quantity: i32,
    target_id: Option<u64>,
) -> (Vec<(u32, i32)>, Vec<(i32, i32)>) {
    let is_selector = material_id >= 481000 && material_id <= 481020;

    let is_hero_selector = material_id == 481022;
    if is_hero_selector && target_id.is_some() {
        return (vec![], vec![]);
    }

    if is_selector && target_id.is_some() {
        let (items, currencies) = get_rewards(material_id);
        let target_idx = target_id.unwrap() as usize;

        if let Some(item) = items.get(target_idx) {
            (vec![(item.0, item.1 * quantity)], vec![])
        } else if let Some(currency) = currencies.get(target_idx) {
            (vec![], vec![(currency.0, currency.1 * quantity)])
        } else {
            tracing::warn!(
                "Invalid target_id {} for selector item {}",
                target_idx,
                material_id
            );
            (vec![], vec![])
        }
    } else if target_id.unwrap_or(0) > 0 {
        let target_id_val = target_id.unwrap();
        (vec![(target_id_val as u32, quantity)], vec![])
    } else {
        let game_data = data::exceldb::get();
        let item_cfg = game_data.item.get(material_id as i32);

        if let Some(cfg) = item_cfg {
            if let Some((items, currencies)) = parse_item(&cfg.effect) {
                let final_items: Vec<(u32, i32)> = items
                    .iter()
                    .map(|(id, amt)| (*id, amt * quantity))
                    .collect();
                let final_currencies: Vec<(i32, i32)> = currencies
                    .iter()
                    .map(|(id, amt)| (*id, amt * quantity))
                    .collect();
                (final_items, final_currencies)
            } else {
                let (items, currencies) = get_rewards(material_id);
                let final_items = if items.len() > 1 {
                    let mut rng = thread_rng();
                    let mut selected = Vec::new();
                    for _ in 0..quantity {
                        if let Some(random_item) = items.choose(&mut rng) {
                            selected.push(*random_item);
                        }
                    }
                    selected
                } else {
                    items
                        .iter()
                        .map(|(id, amt)| (*id, amt * quantity))
                        .collect()
                };
                let final_currencies: Vec<(i32, i32)> = currencies
                    .iter()
                    .map(|(id, amt)| (*id, amt * quantity))
                    .collect();
                (final_items, final_currencies)
            }
        } else {
            tracing::warn!("Item {} not found in game data", material_id);
            (vec![], vec![])
        }
    }
}
