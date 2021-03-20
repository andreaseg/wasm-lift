@echo off
cd lift
cargo build --release
cd ..

cd lift_wasm
wasm-pack build --release
cd ..

cd www
npm install
cd ..