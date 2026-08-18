CREATE TABLE "book_notes" (
  "id" uuid PRIMARY KEY,
  "book" uuid NOT NULL REFERENCES "books" ("id") ON DELETE CASCADE,
  -- Copied from the owning book on insert, same as book_labels: the write path
  -- checks ownership against this column instead of joining books every time.
  "user" uuid NOT NULL REFERENCES "users" ("id") ON DELETE CASCADE,
  "text" TEXT NOT NULL,
  -- Optional: a note about the book as a whole carries no page.
  "page" INTEGER,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  "updated_at" TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Serves the book detail view: this book's notes, newest first.
CREATE INDEX "book_notes_book_created_at_idx"
  ON "book_notes" ("book", "created_at" DESC);

-- Serves the "has notes" badge on covers: every noted book of one user, one
-- range scan per list request.
CREATE INDEX "book_notes_user_book_idx"
  ON "book_notes" ("user", "book");
