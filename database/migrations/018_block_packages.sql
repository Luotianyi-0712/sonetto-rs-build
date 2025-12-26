-- Block packages (purchased/unlocked packages)
CREATE TABLE IF NOT EXISTS user_block_packages (
    user_id INTEGER NOT NULL,
    block_package_id INTEGER NOT NULL,
    unused_block_ids TEXT NOT NULL DEFAULT '[]',
    used_block_ids TEXT NOT NULL DEFAULT '[]',
    PRIMARY KEY (user_id, block_package_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Special blocks (individual special blocks)
CREATE TABLE IF NOT EXISTS user_special_blocks (
    user_id INTEGER NOT NULL,
    block_id INTEGER NOT NULL,
    create_time INTEGER NOT NULL,
    PRIMARY KEY (user_id, block_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Placed blocks on the map
CREATE TABLE IF NOT EXISTS user_blocks (
    user_id INTEGER NOT NULL,
    block_id INTEGER NOT NULL,
    x INTEGER NOT NULL DEFAULT 0,
    y INTEGER NOT NULL DEFAULT 0,
    rotate INTEGER NOT NULL DEFAULT 0,
    water_type INTEGER NOT NULL DEFAULT 0,
    block_color INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, block_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Roads connecting buildings
CREATE TABLE IF NOT EXISTS user_roads (
    user_id INTEGER NOT NULL,
    id INTEGER NOT NULL,
    from_type INTEGER NOT NULL DEFAULT 0,
    to_type INTEGER NOT NULL DEFAULT 0,
    road_points TEXT NOT NULL DEFAULT '[]',
    critter_uid INTEGER NOT NULL DEFAULT 0,
    building_uid INTEGER NOT NULL DEFAULT 0,
    building_define_id INTEGER NOT NULL DEFAULT 0,
    skin_id INTEGER NOT NULL DEFAULT 0,
    block_clean_type INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Room state tracking
CREATE TABLE IF NOT EXISTS user_room_state (
    user_id INTEGER PRIMARY KEY,
    is_reset BOOLEAN NOT NULL DEFAULT 0,
    last_reset_time INTEGER,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_user_block_packages ON user_block_packages(user_id);
CREATE INDEX IF NOT EXISTS idx_user_special_blocks ON user_special_blocks(user_id);
CREATE INDEX IF NOT EXISTS idx_user_blocks ON user_blocks(user_id);
CREATE INDEX IF NOT EXISTS idx_user_roads ON user_roads(user_id);
