-- `books` only has *partial* indexes (ISBN / source id, each with a
-- `WHERE ... IS NOT NULL`), none of which can serve a bare "all books for this
-- user" filter — that sequential-scans today.
-- IF NOT EXISTS: the index was created by hand on the dev database before this
-- migration existed, and a migration that cannot run leaves the app unable to start.
CREATE INDEX IF NOT EXISTS "books_user_idx" ON "books" ("user");
