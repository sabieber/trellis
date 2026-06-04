CREATE TYPE "reading_goal_type" AS ENUM ('books', 'pages');

CREATE TYPE "reading_goal_timeframe" AS ENUM ('year', 'month', 'week');

CREATE TABLE "reading_goals" (
    "id" uuid PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    "user_id" uuid NOT NULL REFERENCES "users" ("id"),
    "goal_type" "reading_goal_type" NOT NULL,
    "timeframe" "reading_goal_timeframe" NOT NULL,
    "target" INTEGER NOT NULL CHECK ("target" > 0),
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT now()
);
