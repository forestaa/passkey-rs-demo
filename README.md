# Passkey Demo
This is a demo of [passkey-rs](https://github.com/1Password/passkey-rs).

# Development
## Server
Create TLS certicates at localhost.
```
mkdir server/certs && cd server/certs
mkcert --install
mkcert passkey-demo.localhost 127.0.0.1 ::1
```

Then run server.
```
cd server
cargo run
```

## Client
Build wasm library.
```
cd client
wasm-pack build -t nodejs
```
