CREATE TYPE "label_kind" AS ENUM ('genre', 'tag');

CREATE TABLE "book_labels" (
  "book" uuid NOT NULL REFERENCES "books" ("id") ON DELETE CASCADE,
  -- Copied from the owning book on insert (books.user is immutable — there is
  -- no transfer-ownership operation), so the by-user lookups below are a range
  -- scan instead of one index probe per book.
  "user" uuid NOT NULL REFERENCES "users" ("id") ON DELETE CASCADE,
  "kind" "label_kind" NOT NULL,
  "label" TEXT NOT NULL,
  "added_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY ("book", "kind", "label")
);

-- "Mystery" and "mystery" are one label on one book; the primary key alone
-- would let both through.
CREATE UNIQUE INDEX "book_labels_book_kind_lower_label_idx"
  ON "book_labels" ("book", "kind", lower("label"));

-- Serves the suggestion lookup and phase 2's facet counts, both scoped by user
-- and grouped by label.
CREATE INDEX "book_labels_user_kind_label_idx"
  ON "book_labels" ("user", "kind", "label");
