-- AntSol Indexer DB Schema
CREATE TABLE IF NOT EXISTS packages (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    author TEXT NOT NULL,
    description TEXT,
    repository TEXT,
    homepage TEXT,
    total_downloads BIGINT DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS versions (
    id SERIAL PRIMARY KEY,
    package_id INTEGER NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
    version TEXT NOT NULL,
    ipfs_hash TEXT NOT NULL,
    downloads BIGINT DEFAULT 0,
    published_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(package_id, version)
);

CREATE TABLE IF NOT EXISTS events (
    id SERIAL PRIMARY KEY,
    event_type TEXT NOT NULL,
    package_name TEXT NOT NULL,
    version TEXT,
    transaction_signature TEXT NOT NULL UNIQUE,
    slot BIGINT NOT NULL,
    block_time TIMESTAMPTZ
);

-- Add created_at column to events if it doesn't exist
DO $$ 
BEGIN 
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'events' AND column_name = 'created_at'
    ) THEN
        ALTER TABLE events ADD COLUMN created_at TIMESTAMPTZ DEFAULT NOW();
    END IF;
END $$;

-- Indexer state for tracking progress
CREATE TABLE IF NOT EXISTS indexer_state (
    id SERIAL PRIMARY KEY,
    last_processed_slot BIGINT NOT NULL,
    last_processed_block_time TIMESTAMPTZ,
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    status TEXT DEFAULT 'running',
    error_count INTEGER DEFAULT 0,
    last_error TEXT
);

-- Insert initial state if not exists
INSERT INTO indexer_state (id, last_processed_slot) 
VALUES (1, 0) 
ON CONFLICT (id) DO NOTHING;

-- Create indexes (check if table/columns exist first)
CREATE INDEX IF NOT EXISTS idx_packages_name ON packages(name);
CREATE INDEX IF NOT EXISTS idx_packages_total_downloads ON packages(total_downloads);

-- Only create these if columns exist
DO $$ 
BEGIN 
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'packages' AND column_name = 'created_at') THEN
        CREATE INDEX IF NOT EXISTS idx_packages_created_at ON packages(created_at);
    END IF;
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'packages' AND column_name = 'updated_at') THEN
        CREATE INDEX IF NOT EXISTS idx_packages_updated_at ON packages(updated_at);
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_versions_package_id ON versions(package_id);
CREATE INDEX IF NOT EXISTS idx_versions_version ON versions(version);
CREATE INDEX IF NOT EXISTS idx_versions_downloads ON versions(downloads);

CREATE INDEX IF NOT EXISTS idx_events_package_name ON events(package_name);
CREATE INDEX IF NOT EXISTS idx_events_event_type ON events(event_type);
CREATE INDEX IF NOT EXISTS idx_events_slot ON events(slot);
CREATE INDEX IF NOT EXISTS idx_events_block_time ON events(block_time);

-- Create index on events.created_at only if column exists
DO $$ 
BEGIN 
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'events' AND column_name = 'created_at'
    ) THEN
        CREATE INDEX IF NOT EXISTS idx_events_created_at ON events(created_at);
    END IF;
END $$;
