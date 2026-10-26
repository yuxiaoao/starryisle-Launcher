CREATE TABLE IF NOT EXISTS screenshot_notes (
    instance_id TEXT NOT NULL,
    file_name TEXT NOT NULL,
    note TEXT NOT NULL,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (instance_id, file_name)
);
