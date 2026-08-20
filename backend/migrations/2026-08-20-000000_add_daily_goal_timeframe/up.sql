-- A daily goal is not a reporting period like the others: it is the barrier the
-- reading streak has to clear on a day. It is only allowed for page goals.
ALTER TYPE "reading_goal_timeframe" ADD VALUE IF NOT EXISTS 'day';
