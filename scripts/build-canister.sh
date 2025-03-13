#!/bin/bash

source .env

set -e

echo -e "\nBuilding canister..."

TELEGRAM_SECRET_TOKEN=$TELEGRAM_SECRET_TOKEN \
cargo build --target wasm32-unknown-unknown --release -p backend --locked
candid-extractor target/wasm32-unknown-unknown/release/backend.wasm > src/backend/backend.did

echo -e "\nDone!\n"
