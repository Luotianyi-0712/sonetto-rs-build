use sonettobuf::{BeginRoundOper, CardInfo};

pub fn generate_auto_opers(deck: &[CardInfo]) -> Vec<BeginRoundOper> {
    let mut opers = Vec::new();

    // Pick first 3 cards (simple baseline)
    for (idx, _card) in deck.iter().take(3).enumerate() {
        let card_index = idx as i32;

        // 1) Select card
        opers.push(BeginRoundOper {
            oper_type: Some(2), // select card
            param1: Some(card_index),
            param2: None,
            to_id: None,
            param3: None,
        });

        // 2) Play card
        opers.push(BeginRoundOper {
            oper_type: Some(1), // play card
            param1: Some(card_index),
            param2: None,
            to_id: Some(0), // auto target
            param3: None,
        });
    }

    // 3) End turn
    opers.push(BeginRoundOper {
        oper_type: Some(4),
        param1: None,
        param2: None,
        to_id: None,
        param3: None,
    });

    opers
}
