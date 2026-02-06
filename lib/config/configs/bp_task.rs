// Auto-generated from JSON data
// Do not edit manually

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpTask {
    #[serde(rename = "bonusMail")]
    pub bonus_mail: i32,
    #[serde(rename = "bonusScore")]
    pub bonus_score: i32,
    #[serde(rename = "bonusScoreTimes")]
    pub bonus_score_times: i32,
    #[serde(rename = "bpId")]
    pub bp_id: i32,
    pub desc: String,
    #[serde(rename = "endTime")]
    pub end_time: String,
    pub id: i32,
    #[serde(rename = "isOnline")]
    pub is_online: i32,
    #[serde(rename = "jumpId")]
    pub jump_id: i32,
    #[serde(rename = "listenerParam")]
    pub listener_param: String,
    #[serde(rename = "listenerType")]
    pub listener_type: String,
    #[serde(rename = "loopType")]
    pub loop_type: i32,
    #[serde(rename = "maxProgress")]
    pub max_progress: i32,
    #[serde(rename = "minType")]
    pub min_type: String,
    pub name: String,
    #[serde(rename = "newbieTask")]
    pub newbie_task: bool,
    #[serde(rename = "openLimit")]
    pub open_limit: String,
    pub params: String,
    pub prepose: String,
    #[serde(rename = "sortId")]
    pub sort_id: i32,
    #[serde(rename = "startTime")]
    pub start_time: String,
    #[serde(rename = "turnbackTask")]
    pub turnback_task: bool,
}
use std::collections::HashMap;

pub struct BpTaskTable {
    records: Vec<BpTask>,
    by_id: HashMap<i32, usize>,
}

impl BpTaskTable {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&json)?;

        let records: Vec<BpTask> = if let Some(array) = value.as_array() {
            if array.len() >= 2 && array[1].is_array() {
                serde_json::from_value(array[1].clone())?
            } else {
                serde_json::from_value(value)?
            }
        } else {
            serde_json::from_value(value)?
        };

        let mut by_id = HashMap::with_capacity(records.len());

        for (idx, record) in records.iter().enumerate() {
            by_id.insert(record.id, idx);
        }

        Ok(Self {
            records,
            by_id,
        })
    }

    #[inline]
    pub fn get(&self, id: i32) -> Option<&BpTask> {
        self.by_id.get(&id).map(|&i| &self.records[i])
    }

    #[inline]
    pub fn all(&self) -> &[BpTask] {
        &self.records
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, BpTask> {
        self.records.iter()
    }

    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}