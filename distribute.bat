@echo off
cd www 
if exist dist rmdir dist /q /s
cmd /c "npm run dist"

cd dist
for /r %%i in (*.wasm) do (
    echo "%%i"
    rem wasm-opt %%i -Oz
)
