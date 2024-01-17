#!/bin/bash
docker run -d --rm --name mydb -p 5432:5432 -e POSTGRES_PASSWORD=welcome -v ./pg_data:/var/lib/postgresql/data postgres
