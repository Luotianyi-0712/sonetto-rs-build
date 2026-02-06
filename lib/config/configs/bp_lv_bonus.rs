// Auto-generated from JSON data
// Do not edit manually

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpLvBonus {
    #[serde(rename = "bpId")]
    pub bp_id: i32,
    #[serde(rename = "freeBonus")]
    pub free_bonus: String,
    #[serde(rename = "keyBonus")]
    pub key_bonus: i32,
    pub level: i32,
    #[serde(rename = "payBonus")]
    pub pay_bonus: String,
    #[serde(rename = "selfSelectPayBonus")]
    pub self_select_pay_bonus: String,
    #[serde(rename = "selfSelectPayItem")]
    pub self_select_pay_item: String,
    #[serde(rename = "spFreeBonus")]
    pub sp_free_bonus: String,
    #[serde(rename = "spPayBonus")]
    pub sp_pay_bonus: String,
}
pub struct BpLvBonusTable {
    records: Vec<BpLvBonus>,
}

impl BpLvBonusTable {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&json)?;

        let records: Vec<BpLvBonus> = if let Some(array) = value.as_array() {
            if array.len() >= 2 && array[1].is_array() {
                serde_json::from_value(array[1].clone())?
            } else {
                serde_json::from_value(value)?
            }
        } else {
            serde_json::from_value(value)?
        };

        Ok(Self {
            records,
        })
    }

    #[inline]
    pub fn all(&self) -> &[BpLvBonus] {
        &self.records
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, BpLvBonus> {
        self.records.iter()
    }

    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}