ALTER TABLE "books" ADD COLUMN "open_library_id" text;

CREATE UNIQUE INDEX "books_user_open_library_id_key"
  ON "books" ("user", "open_library_id") WHERE "open_library_id" IS NOT NULL;
