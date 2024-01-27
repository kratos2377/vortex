-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS "user_friends" (
  "id" uuid PRIMARY KEY,
  "following_user_id" integer,
  "followed_user_id" integer,
  "created_at" timestamp
);

CREATE TABLE IF NOT EXISTS "users" (
  "id" uuid PRIMARY KEY,
  "password" varchar,
  "username" varchar,
  "first_name" varchar,
  "last_name" varchar,
  "score" integer,
  "created_at" timestamp,
  "updated_at" timestamp
);

CREATE TABLE IF NOT EXISTS "users_wallet_keys" (
  "id" uuid PRIMARY KEY,
  "solana_wallet_address" varchar,
  "user_id" uuid
);

ALTER TABLE "users_wallet_keys" ADD FOREIGN KEY ("user_id") REFERENCES "users" ("id");

ALTER TABLE "user_friends" ADD FOREIGN KEY ("following_user_id") REFERENCES "users" ("id");

ALTER TABLE "user_friends" ADD FOREIGN KEY ("followed_user_id") REFERENCES "users" ("id");
