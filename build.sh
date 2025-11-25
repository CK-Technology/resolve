#!/bin/bash
export DATABASE_URL=postgresql://resolve:resolve@localhost:5432/resolve
cargo build "$@"