DROP INDEX IF EXISTS "books_user_isbn13_idx";
DROP INDEX IF EXISTS "books_user_isbn10_idx";

CREATE UNIQUE INDEX "books_user_isbn13_key"
  ON "books" ("user", "isbn13") WHERE "isbn13" IS NOT NULL;
CREATE UNIQUE INDEX "books_user_isbn10_key"
  ON "books" ("user", "isbn10") WHERE "isbn10" IS NOT NULL;
