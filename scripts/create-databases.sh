#!/bin/bash

set -e

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    CREATE USER demo WITH PASSWORD 'secret';
    CREATE DATABASE vortex;
    GRANT ALL PRIVILEGES ON DATABASE vortex TO demo;
    \c vortex
    GRANT ALL ON SCHEMA public TO demo;
EOSQL