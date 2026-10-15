-- Drop the Paddle billing schema inherited from the template.
--
-- marketplace.plans, marketplace.subscriptions, marketplace.paddle_customers
-- and marketplace.paddle_webhook_events had no reader and no writer besides a
-- boot seed of two placeholder plans; this installation bills nothing through
-- Paddle. The schema held nothing else.
DROP SCHEMA IF EXISTS marketplace CASCADE;
