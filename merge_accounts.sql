-- Merge two accounts into one.
-- Fill in the IDs below, then run:
--   sqlite3 data/whistbook.db < merge_accounts.sql
--
-- To find IDs:
--   sqlite3 data/whistbook.db "SELECT id, email FROM login;"

-- !! SET THESE !!
-- Account to KEEP (the one that survives)
-- Account to REMOVE (will be deleted after merge)
-- Replace 0 with the actual IDs.

-- Example:
--   .param set @keep_id   3
--   .param set @remove_id 7

.param set @keep_id   6
.param set @remove_id 4

-- Safety check: abort if IDs are unset or equal
SELECT CASE
    WHEN @keep_id = 0 OR @remove_id = 0 THEN RAISE(ABORT, 'Set @keep_id and @remove_id before running')
    WHEN @keep_id = @remove_id           THEN RAISE(ABORT, '@keep_id and @remove_id must be different')
END;

BEGIN;

-- Move plays from the removed account to the kept account,
-- skipping games where the kept account is already a player
-- (both accounts in the same game — kept account's entry wins).
UPDATE plays
SET login_id = @keep_id
WHERE login_id = @remove_id
  AND game_id NOT IN (
      SELECT game_id FROM plays WHERE login_id = @keep_id
  );

-- Drop any remaining plays for the removed account
-- (these are games where both accounts were already present).
DELETE FROM plays WHERE login_id = @remove_id;

-- Drop the removed account's rating (will be recomputed on next deal/login).
DELETE FROM rating WHERE login_id = @remove_id;

-- Delete the removed login.
DELETE FROM login WHERE id = @remove_id;

COMMIT;

-- Verify
SELECT 'Remaining logins:';
SELECT id, email FROM login;
SELECT 'plays for kept account:';
SELECT * FROM plays WHERE login_id = @keep_id;
