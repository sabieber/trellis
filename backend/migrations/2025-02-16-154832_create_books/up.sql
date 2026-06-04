CREATE TABLE "books" (
  "id" uuid PRIMARY KEY NOT NULL,
  "user" uuid NOT NULL REFERENCES "users" ("id"),
  "title" text,
  "author" text,
  "isbn13" text,
  "isbn10" text,
  "google_books_id" text,
  "added_at" TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- A book exists once per user. These partial unique indexes backstop the
-- application-level identity ladder against races. The title+author fallback is
-- enforced in application logic only.
CREATE UNIQUE INDEX "books_user_google_books_id_key"
  ON "books" ("user", "google_books_id") WHERE "google_books_id" IS NOT NULL;
CREATE UNIQUE INDEX "books_user_isbn13_key"
  ON "books" ("user", "isbn13") WHERE "isbn13" IS NOT NULL;
CREATE UNIQUE INDEX "books_user_isbn10_key"
  ON "books" ("user", "isbn10") WHERE "isbn10" IS NOT NULL;
