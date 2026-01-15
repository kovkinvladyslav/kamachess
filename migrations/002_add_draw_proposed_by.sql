ALTER TABLE games ADD COLUMN draw_proposed_by INTEGER;
CREATE INDEX IF NOT EXISTS idx_games_draw_proposed ON games(id) WHERE draw_proposed_by IS NOT NULL;
