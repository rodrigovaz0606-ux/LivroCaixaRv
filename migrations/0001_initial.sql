CREATE TABLE IF NOT EXISTS yearly_balances (
    year INTEGER PRIMARY KEY CHECK (year BETWEEN 1900 AND 2200),
    opening_balance_cents INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_date TEXT NOT NULL,
    description TEXT NOT NULL CHECK (length(trim(description)) > 0),
    document TEXT NOT NULL DEFAULT '',
    category TEXT NOT NULL DEFAULT '',
    entry_type TEXT NOT NULL CHECK (entry_type IN ('entrada', 'saida')),
    amount_cents INTEGER NOT NULL CHECK (amount_cents > 0),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_entries_date ON entries(entry_date);

