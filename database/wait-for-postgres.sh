#!/bin/bash
# wait-for-postgres.sh

set -e

until psql $DATABASE_URL -c '\q'; do
  >&2 echo "Postgres is unavailable - sleeping"
  sleep 1
done

>&2 echo "Postgres is up - executing command"
diesel migration run
