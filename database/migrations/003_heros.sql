-- Main heroes table
CREATE TABLE IF NOT EXISTS heroes (
    uid INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    hero_id INTEGER NOT NULL,
    create_time INTEGER NOT NULL,
    level INTEGER NOT NULL,
    exp INTEGER NOT NULL,
    rank INTEGER NOT NULL,
    breakthrough INTEGER NOT NULL,
    skin INTEGER NOT NULL,
    faith INTEGER NOT NULL,
    active_skill_level INTEGER NOT NULL,
    ex_skill_level INTEGER NOT NULL,
    is_new BOOLEAN NOT NULL DEFAULT 1,
    talent INTEGER NOT NULL DEFAULT 0,
    default_equip_uid INTEGER NOT NULL DEFAULT 0,
    duplicate_count INTEGER NOT NULL DEFAULT 0,
    use_talent_template_id INTEGER NOT NULL DEFAULT 0,
    talent_style_unlock INTEGER NOT NULL DEFAULT 0,
    talent_style_red INTEGER NOT NULL DEFAULT 0,
    is_favor BOOLEAN NOT NULL DEFAULT 0,
    destiny_rank INTEGER NOT NULL DEFAULT 0,
    destiny_level INTEGER NOT NULL DEFAULT 0,
    destiny_stone INTEGER NOT NULL DEFAULT 0,
    red_dot INTEGER NOT NULL DEFAULT 0,
    extra_str TEXT NOT NULL DEFAULT '',

    -- Base attributes
    base_hp INTEGER NOT NULL,
    base_attack INTEGER NOT NULL,
    base_defense INTEGER NOT NULL,
    base_mdefense INTEGER NOT NULL,
    base_technic INTEGER NOT NULL,
    base_multi_hp_idx INTEGER NOT NULL DEFAULT 0,
    base_multi_hp_num INTEGER NOT NULL DEFAULT 0,

    -- Ex attributes
    ex_cri INTEGER NOT NULL DEFAULT 0,
    ex_recri INTEGER NOT NULL DEFAULT 0,
    ex_cri_dmg INTEGER NOT NULL DEFAULT 0,
    ex_cri_def INTEGER NOT NULL DEFAULT 0,
    ex_add_dmg INTEGER NOT NULL DEFAULT 0,
    ex_drop_dmg INTEGER NOT NULL DEFAULT 0,

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_heroes_user_id ON heroes(user_id);
CREATE INDEX IF NOT EXISTS idx_heroes_hero_id ON heroes(hero_id);

-- Hero passive skill levels
CREATE TABLE IF NOT EXISTS hero_passive_skill_levels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hero_uid INTEGER NOT NULL,
    skill_index INTEGER NOT NULL,
    level INTEGER NOT NULL,
    FOREIGN KEY (hero_uid) REFERENCES heroes(uid) ON DELETE CASCADE,
    UNIQUE(hero_uid, skill_index)
);

CREATE INDEX IF NOT EXISTS idx_hero_passive_skills_uid ON hero_passive_skill_levels(hero_uid);

-- Hero voices (unlocked)
CREATE TABLE IF NOT EXISTS hero_voices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hero_uid INTEGER NOT NULL,
    voice_id INTEGER NOT NULL,
    FOREIGN KEY (hero_uid) REFERENCES heroes(uid) ON DELETE CASCADE,
    UNIQUE(hero_uid, voice_id)
);

CREATE INDEX IF NOT EXISTS idx_hero_voices_uid ON hero_voices(hero_uid);

-- Hero voices heard (played/listened to)
CREATE TABLE IF NOT EXISTS hero_voices_heard (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hero_uid INTEGER NOT NULL,
    voice_id INTEGER NOT NULL,
    FOREIGN KEY (hero_uid) REFERENCES heroes(uid) ON DELETE CASCADE,
    UNIQUE(hero_uid, voice_id)
);

CREATE INDEX IF NOT EXISTS idx_hero_voices_heard_uid ON hero_voices_heard(hero_uid);

-- Hero skins
CREATE TABLE IF NOT EXISTS hero_skins (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hero_uid INTEGER NOT NULL,
    skin INTEGER NOT NULL,
    expire_sec INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (hero_uid) REFERENCES heroes(uid) ON DELETE CASCADE,
    UNIQUE(hero_uid, skin)
);

CREATE INDEX IF NOT EXISTS idx_hero_skins_uid ON hero_skins(hero_uid);

-- Hero special attributes (29 fields from HeroSpAttribute proto)
CREATE TABLE IF NOT EXISTS hero_sp_attrs (
    hero_uid INTEGER PRIMARY KEY,
    revive INTEGER NOT NULL DEFAULT 0,
    heal INTEGER NOT NULL DEFAULT 0,
    absorb INTEGER NOT NULL DEFAULT 0,
    defense_ignore INTEGER NOT NULL DEFAULT 0,
    clutch INTEGER NOT NULL DEFAULT 0,
    final_add_dmg INTEGER NOT NULL DEFAULT 0,
    final_drop_dmg INTEGER NOT NULL DEFAULT 0,
    normal_skill_rate INTEGER NOT NULL DEFAULT 0,
    play_add_rate INTEGER NOT NULL DEFAULT 0,
    play_drop_rate INTEGER NOT NULL DEFAULT 0,
    dizzy_resistances INTEGER NOT NULL DEFAULT 0,
    sleep_resistances INTEGER NOT NULL DEFAULT 0,
    petrified_resistances INTEGER NOT NULL DEFAULT 0,
    frozen_resistances INTEGER NOT NULL DEFAULT 0,
    disarm_resistances INTEGER NOT NULL DEFAULT 0,
    forbid_resistances INTEGER NOT NULL DEFAULT 0,
    seal_resistances INTEGER NOT NULL DEFAULT 0,
    cant_get_exskill_resistances INTEGER NOT NULL DEFAULT 0,
    del_ex_point_resistances INTEGER NOT NULL DEFAULT 0,
    stress_up_resistances INTEGER NOT NULL DEFAULT 0,
    control_resilience INTEGER NOT NULL DEFAULT 0,
    del_ex_point_resilience INTEGER NOT NULL DEFAULT 0,
    stress_up_resilience INTEGER NOT NULL DEFAULT 0,
    charm_resistances INTEGER NOT NULL DEFAULT 0,
    rebound_dmg INTEGER NOT NULL DEFAULT 0,
    extra_dmg INTEGER NOT NULL DEFAULT 0,
    reuse_dmg INTEGER NOT NULL DEFAULT 0,
    big_skill_rate INTEGER NOT NULL DEFAULT 0,
    clutch_dmg INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (hero_uid) REFERENCES heroes(uid) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_hero_sp_attrs_uid ON hero_sp_attrs(hero_uid);

-- Hero equipment attributes (per equipment piece)
CREATE TABLE IF NOT EXISTS hero_equip_attributes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hero_uid INTEGER NOT NULL,
    equip_id INTEGER NOT NULL,
    hp INTEGER NOT NULL DEFAULT 0,
    attack INTEGER NOT NULL DEFAULT 0,
    defense INTEGER NOT NULL DEFAULT 0,
    mdefense INTEGER NOT NULL DEFAULT 0,
    technic INTEGER NOT NULL DEFAULT 0,
    multi_hp_idx INTEGER NOT NULL DEFAULT 0,
    multi_hp_num INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (hero_uid) REFERENCES heroes(uid) ON DELETE CASCADE,
    UNIQUE(hero_uid, equip_id)
);

CREATE INDEX IF NOT EXISTS idx_hero_equip_attrs_uid ON hero_equip_attributes(hero_uid);

-- Hero item unlocks
CREATE TABLE IF NOT EXISTS hero_item_unlocks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hero_uid INTEGER NOT NULL,
    item_id INTEGER NOT NULL,
    FOREIGN KEY (hero_uid) REFERENCES heroes(uid) ON DELETE CASCADE,
    UNIQUE(hero_uid, item_id)
);

CREATE INDEX IF NOT EXISTS idx_hero_item_unlocks_uid ON hero_item_unlocks(hero_uid);

-- Hero talent cubes (active/equipped)
CREATE TABLE IF NOT EXISTS hero_talent_cubes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hero_uid INTEGER NOT NULL,
    cube_id INTEGER NOT NULL,
    direction INTEGER NOT NULL,
    pos_x INTEGER NOT NULL,
    pos_y INTEGER NOT NULL,
    FOREIGN KEY (hero_uid) REFERENCES heroes(uid) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_hero_talent_cubes_uid ON hero_talent_cubes(hero_uid);

-- Hero talent templates (saved loadouts)
CREATE TABLE IF NOT EXISTS hero_talent_templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hero_uid INTEGER NOT NULL,
    template_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    style INTEGER NOT NULL,
    FOREIGN KEY (hero_uid) REFERENCES heroes(uid) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_hero_talent_templates_uid ON hero_talent_templates(hero_uid);

-- Hero talent template cubes (cubes within saved templates)
CREATE TABLE IF NOT EXISTS hero_talent_template_cubes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    template_row_id INTEGER NOT NULL,
    cube_id INTEGER NOT NULL,
    direction INTEGER NOT NULL,
    pos_x INTEGER NOT NULL,
    pos_y INTEGER NOT NULL,
    FOREIGN KEY (template_row_id) REFERENCES hero_talent_templates(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_hero_template_cubes_template ON hero_talent_template_cubes(template_row_id);

-- Hero destiny stone unlocks
CREATE TABLE IF NOT EXISTS hero_destiny_stone_unlocks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hero_uid INTEGER NOT NULL,
    stone_id INTEGER NOT NULL,
    FOREIGN KEY (hero_uid) REFERENCES heroes(uid) ON DELETE CASCADE,
    UNIQUE(hero_uid, stone_id)
);

CREATE INDEX IF NOT EXISTS idx_hero_destiny_unlocks_uid ON hero_destiny_stone_unlocks(hero_uid);

-- User-level hero data (not per-hero instance)
-- All skins owned by user (account-wide)
CREATE TABLE IF NOT EXISTS hero_all_skins (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    skin_id INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, skin_id)
);

CREATE INDEX IF NOT EXISTS idx_hero_all_skins_user ON hero_all_skins(user_id);

-- Hero birthday celebration count
CREATE TABLE IF NOT EXISTS hero_birthday_info (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    hero_id INTEGER NOT NULL,
    birthday_count INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, hero_id)
);

CREATE INDEX IF NOT EXISTS idx_hero_birthday_user ON hero_birthday_info(user_id);

-- Touch/interaction count limit
CREATE TABLE IF NOT EXISTS hero_touch_count (
    user_id INTEGER PRIMARY KEY,
    touch_count_left INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS hero_talent_styles (
    hero_uid INTEGER NOT NULL,
    style_id INTEGER NOT NULL,
    PRIMARY KEY (hero_uid, style_id)
);
