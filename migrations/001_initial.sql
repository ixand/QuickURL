CREATE TABLE IF NOT EXISTS urls (
    id TEXT PRIMARY KEY,
    token TEXT UNIQUE NOT NULL,
    original_url TEXT NOT NULL,
    title TEXT,
    created_at DATETIME NOT NULL,
    expires_at DATETIME NOT NULL,
    click_count INTEGER DEFAULT 0
);

-- Create index for faster token lookups
CREATE INDEX IF NOT EXISTS idx_urls_token ON urls(token);
CREATE INDEX IF NOT EXISTS idx_urls_created_at ON urls(created_at);
CREATE INDEX IF NOT EXISTS idx_urls_expires_at ON urls(expires_at);
