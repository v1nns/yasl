# yasl (Yet Another Spotify Listener)

Combine the useful with the pleasant and you get this: an attempt to practice a
little bit more about Rust and solve the problem about how to script a way to
listen to DBus interface and check it out if there is any media playing.

# Quick start

First thing, you must compile it locally:

```bash
# Clone repository
git clone https://github.com/v1nns/yasl
cd yasl
# compile it using cargo
cargo build --release
# OPTIONAL
cd target/release && cp ./yasl ~/.local/bin
```

Then, you can use it with Polybar, just like this:

    [module/spotifyd]
    type = custom/script
    label-maxlen = 42
    label-ellipsis = true

    format = <label>

    tail = true
    exec = yasl
