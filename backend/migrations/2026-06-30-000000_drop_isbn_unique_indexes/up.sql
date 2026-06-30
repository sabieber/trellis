DROP INDEX IF EXISTS "books_user_isbn13_key";
DROP INDEX IF EXISTS "books_user_isbn10_key";

CREATE INDEX "books_user_isbn13_idx"
  ON "books" ("user", "isbn13") WHERE "isbn13" IS NOT NULL;
CREATE INDEX "books_user_isbn10_idx"
  ON "books" ("user", "isbn10") WHERE "isbn10" IS NOT NULL;
