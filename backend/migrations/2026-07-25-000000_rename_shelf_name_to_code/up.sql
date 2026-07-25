-- The old "name" doubled as an import-resolution key (GoodReads shelf names map
-- to it). Rename it to "code" to reflect that, and add an optional display
-- "name" the user can edit; when null, the code is shown instead.
ALTER TABLE "shelves" RENAME COLUMN "name" TO "code";
ALTER TABLE "shelves" ADD COLUMN "name" TEXT;
