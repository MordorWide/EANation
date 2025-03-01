#!/bin/bash

# Wait if DB_PORT and DB_HOST are set
if [ -n "$DB_PORT" ] && [ -n "$DB_HOST" ]; then
    echo "Waiting for the database ($DB_HOST:$DB_PORT) to be ready..."
    bash -c 'until printf "" 2>>/dev/null >>/dev/tcp/$0/$1; do sleep 1; done' "$DB_HOST" "$DB_PORT"
    echo "Database is ready!"
fi

exec "$@"