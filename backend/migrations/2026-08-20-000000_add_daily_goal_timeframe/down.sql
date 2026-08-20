-- Postgres cannot drop a single enum value, so the type is rebuilt without it.
DELETE FROM "reading_goals" WHERE "timeframe" = 'day';

ALTER TYPE "reading_goal_timeframe" RENAME TO "reading_goal_timeframe_old";
CREATE TYPE "reading_goal_timeframe" AS ENUM ('year', 'month', 'week');
ALTER TABLE "reading_goals"
    ALTER COLUMN "timeframe" TYPE "reading_goal_timeframe"
    USING "timeframe"::text::"reading_goal_timeframe";
DROP TYPE "reading_goal_timeframe_old";
