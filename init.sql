-- -------------------------------------------------------------
-- TablePlus 6.1.8(574)
--
-- https://tableplus.com/
--
-- Database: axum_postgres
-- Generation Time: 2024-11-04 09:48:50.6060
-- -------------------------------------------------------------


DROP TABLE IF EXISTS "public"."tasks";
-- This script only contains the table creation statements and does not fully represent the table in the database. Do not use it as a backup.

-- Sequence and defined type
CREATE SEQUENCE IF NOT EXISTS tasks_id_seq;

-- Table Definition
CREATE TABLE "public"."tasks" (
    "id" int4 NOT NULL DEFAULT nextval('tasks_id_seq'::regclass),
    "name" varchar NOT NULL,
    "priority" int4,
    PRIMARY KEY ("id")
);

