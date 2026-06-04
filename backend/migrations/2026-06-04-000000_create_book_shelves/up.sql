CREATE TABLE "book_shelves" (
  "book" uuid NOT NULL REFERENCES "books" ("id") ON DELETE CASCADE,
  "shelf" uuid NOT NULL REFERENCES "shelves" ("id") ON DELETE CASCADE,
  "added_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY ("book", "shelf")
);

CREATE INDEX "book_shelves_shelf_idx" ON "book_shelves" ("shelf");
