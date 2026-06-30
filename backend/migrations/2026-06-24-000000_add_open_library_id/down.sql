DROP INDEX IF EXISTS "books_user_open_library_id_key";
ALTER TABLE "books" DROP COLUMN IF EXISTS "open_library_id";
