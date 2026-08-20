-- The remaining profile preferences move next to `rating_mode`, so they follow
-- the user instead of the device.
--
-- An empty `locale` means the user never picked one: the browser language keeps
-- answering. `edition_languages` holds the language codes in the order the user
-- picked them, and an empty array means "show every edition".
ALTER TABLE "users" ADD COLUMN "locale" VARCHAR(8) NOT NULL DEFAULT '';
ALTER TABLE "users" ADD COLUMN "edition_languages" TEXT[] NOT NULL DEFAULT '{}';
