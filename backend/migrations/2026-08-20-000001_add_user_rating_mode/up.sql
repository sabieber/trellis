-- How the user reads and writes a rating. The `books.rating` column stays a
-- 1..5 score in both modes: thumbs down writes 1, thumbs up writes 5, and an
-- existing score renders through its tendency.
ALTER TABLE "users" ADD COLUMN "rating_mode" VARCHAR(8) NOT NULL DEFAULT 'stars';
