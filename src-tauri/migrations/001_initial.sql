-- Create the main blocks table
CREATE TABLE IF NOT EXISTS blocks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content TEXT, -- The actual text content of the block (markdown)
    parent_id UUID, -- The block this one is nested under (self-referencing for hierarchy)
    "order" INTEGER NOT NULL, -- The sort order of the block within its parent
    is_page BOOLEAN DEFAULT FALSE, -- True if this block represents a page
    page_title TEXT, -- The title of the page, if is_page is true. Indexed for quick lookup.
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    -- Foreign key constraint for parent-child relationship
    CONSTRAINT fk_parent
      FOREIGN KEY(parent_id)
      REFERENCES blocks(id)
      ON DELETE CASCADE -- If a parent is deleted, its children are too.
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_blocks_parent_id ON blocks(parent_id);
CREATE INDEX IF NOT EXISTS idx_blocks_page_title ON blocks(page_title);

-- Table for audio recordings
CREATE TABLE IF NOT EXISTS audio_recordings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    page_id UUID NOT NULL, -- Which page this recording belongs to
    file_path TEXT NOT NULL, -- Absolute path to the audio file on the user's system
    duration_seconds INTEGER, -- Total duration of the recording
    recorded_at TIMESTAMPTZ DEFAULT NOW(),
    -- Foreign key to the page (which is a block)
    CONSTRAINT fk_page
      FOREIGN KEY(page_id)
      REFERENCES blocks(id)
      ON DELETE CASCADE
);

-- Index for quick lookup of recordings by page
CREATE INDEX IF NOT EXISTS idx_audio_recordings_page_id ON audio_recordings(page_id);

-- Table to link blocks to a specific timestamp in an audio recording
CREATE TABLE IF NOT EXISTS audio_timestamps (
    id SERIAL PRIMARY KEY,
    block_id UUID NOT NULL,
    recording_id UUID NOT NULL,
    timestamp_seconds INTEGER NOT NULL, -- The point in the audio file (in seconds)
    -- Foreign key to the block
    CONSTRAINT fk_block
      FOREIGN KEY(block_id)
      REFERENCES blocks(id)
      ON DELETE CASCADE,
    -- Foreign key to the recording
    CONSTRAINT fk_recording
      FOREIGN KEY(recording_id)
      REFERENCES audio_recordings(id)
      ON DELETE CASCADE,
    -- A block can only have one timestamp per recording
    UNIQUE (block_id, recording_id)
);

-- Index for quickly finding a block's timestamp
CREATE INDEX IF NOT EXISTS idx_audio_timestamps_block_id ON audio_timestamps(block_id);

