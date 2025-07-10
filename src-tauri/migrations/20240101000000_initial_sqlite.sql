-- Create the main blocks table
CREATE TABLE IF NOT EXISTS blocks (
    id TEXT PRIMARY KEY,
    content TEXT, -- The actual text content of the block (markdown)
    parent_id TEXT, -- The block this one is nested under (self-referencing for hierarchy)
    "order" INTEGER NOT NULL, -- The sort order of the block within its parent
    is_page BOOLEAN DEFAULT FALSE, -- True if this block represents a page
    page_title TEXT, -- The title of the page, if is_page is true. Indexed for quick lookup.
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now')),
    -- Foreign key constraint for parent-child relationship
    FOREIGN KEY(parent_id) REFERENCES blocks(id) ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_blocks_parent_id ON blocks(parent_id);
CREATE INDEX IF NOT EXISTS idx_blocks_page_title ON blocks(page_title);

-- Table for audio recordings
CREATE TABLE IF NOT EXISTS audio_recordings (
    id TEXT PRIMARY KEY,
    page_id TEXT NOT NULL, -- Which page this recording belongs to
    file_path TEXT NOT NULL, -- Absolute path to the audio file on the user's system
    duration_seconds INTEGER, -- Total duration of the recording
    recorded_at TEXT DEFAULT (datetime('now')),
    -- Foreign key to the page (which is a block)
    FOREIGN KEY(page_id) REFERENCES blocks(id) ON DELETE CASCADE
);

-- Index for quick lookup of recordings by page
CREATE INDEX IF NOT EXISTS idx_audio_recordings_page_id ON audio_recordings(page_id);

-- Table to link blocks to a specific timestamp in an audio recording
CREATE TABLE IF NOT EXISTS audio_timestamps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    block_id TEXT NOT NULL,
    recording_id TEXT NOT NULL,
    timestamp_seconds INTEGER NOT NULL, -- The point in the audio file (in seconds)
    -- Foreign key to the block
    FOREIGN KEY(block_id) REFERENCES blocks(id) ON DELETE CASCADE,
    -- Foreign key to the recording
    FOREIGN KEY(recording_id) REFERENCES audio_recordings(id) ON DELETE CASCADE,
    -- A block can only have one timestamp per recording
    UNIQUE (block_id, recording_id)
);

-- Index for quickly finding a block's timestamp
CREATE INDEX IF NOT EXISTS idx_audio_timestamps_block_id ON audio_timestamps(block_id);
