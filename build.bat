@echo off
cd lift
cargo build --release
cd ..

cd lift_wasm
wasm-pack build
cd ..

cd www
npm install
cd ..