-- Postgres cannot drop a single enum value, so the type is rebuilt without them.
DELETE FROM "book_labels"
 WHERE "kind" IN ('received_from', 'given_to', 'borrowed_from', 'borrowed_to');

ALTER TYPE "label_kind" RENAME TO "label_kind_old";
CREATE TYPE "label_kind" AS ENUM ('genre', 'tag');
ALTER TABLE "book_labels"
    ALTER COLUMN "kind" TYPE "label_kind"
    USING "kind"::text::"label_kind";
DROP TYPE "label_kind_old";
