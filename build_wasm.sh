cargo build --release -p trc_client --target wasm32-unknown-unknown --features bevy/webgl2
wasm-bindgen --out-name trc_client_wasm \
  --out-dir wasm_output \
  --target web target/wasm32-unknown-unknown/release/trc_client.wasm
