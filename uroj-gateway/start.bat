@echo off
:cleanup 
    kill "$ACCOUNTS_PID"
    kill "$PRODUCTS_PID"
    kill "$REVIEWS_PID"
EXIT /B 0

trap cleanup EXIT
cargo build --bin uroj-api
cargo build --bin uroj-auth
cargo build --bin uroj-runtime
cargo run --bin uroj-runtime
SET ACCOUNTS_PID=%!%
cargo run --bin uroj-api
SET PRODUCTS_PID=%!%
cargo run --bin uroj-auth
SET REVIEWS_PID=%!%
sleep 3
node index.js