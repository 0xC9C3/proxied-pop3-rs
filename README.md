#  proxied-pop3

A small lib with a few modifications to https://github.com/mattnenterprise/rust-pop3
to allow for a proxied socks5 connection to a pop3 server using https://crates.io/crates/fast-socks5.

This is not published to crates.io, so you'll have to clone the repo and use it as a local dependency
or make a crates.io worthy version of it.

This mixes async and sync, so it's not pretty, but it works.

Code example in `src/main.rs`. Copy the .env.example to .env and fill in the details.