#!/usr/bin/env bash

name=code-things-pg
image=postgres
username=lavrenyk
password=kakeepoo
dbname=social
port=5432

# shellcheck disable=SC2053
[[ $(docker ps -f "name=$name" --format '{{.Names}}') == $name ]] || docker run -d \
    --name "$name" \
    -p $port:$port \
    -e POSTGRES_USER=$username \
    -e POSTGRES_PASSWORD=$password \
    -e POSTGRES_DB=$dbname \
    -v "$(pwd)/data:/var/lib/postgresql/data" \
    -v "$(pwd)/initdb.d:/docker-entrypoint-initdb.d" \
    $image