ALTER TABLE "books" ADD COLUMN "rating" SMALLINT CHECK ("rating" >= 1 AND "rating" <= 5);
