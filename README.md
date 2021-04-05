# websocket-serial-server (wssps)

[![Build Status](https://travis-ci.org/DanielJoyce/websocket-serial-server.svg?branch=master)](https://travis-ci.org/DanielJoyce/websocket-serial-server)

Connect to and read / write serial ports over websockets. In Rust

There is a need for a way to interface with hardware for software running in web browsers. WebUSB and the HTML5 serial spec are still immature.

**Alpha, but works for me**

**Currently there is no security, the connection is NOT encrypted**

## Features

1. Written in Rust, so robust and memory safe.
1. All requests and responses are JSON based
1. Supports sending and receiving binary data, as base64 encoded strings
1. Clients can subscribe to multiple ports
1. Clients can write lock ports, so they are the only one
who can send data to it. Writing to a port can not happen
till port is write locked. This prevents corruption
1. Ports are only closed when all clients have closed it
1. Data read from port is broadcast to all clients who opeoned it.
1. Ports are automatically cleaned up if read/write errors occur
1. Opening the same port twice will not cause corruption of data
send to a client ( as seen in SPJS ).
1. Supports port enumeration.
1. simple programming model consisting of threads and event loops, which is fine for dozens of clients and ports.
    1. As the async paradigm in rust matures, will move to that model
1. Simple architecture and code base.

## Documentation

[Documentation](DOCUMENTATION.md)

## Limitations

Currently Websocket-rs is not tokio based, so it spawns a thread per connection.
For having a few clients talk to a 3D printer, CNC machine, or other 
such use case, this is fine. 

Once Websocket-rs moves to tokio, this limitation can be removed

*There is no support for custom protocol or buffer handlers.* That should be handled by client libraries. The purpose of wssps is to simply get data from a serial port to clients and vice-versa.

## Developing

### Dependencies

#### Linux

1. `sudo apt-get install -y libudev-dev libssl-dev pkg-config`

#### Windows

Unknown, help appreciated.

#### OSX

Unknown, help appreciated.


### Vscode-Rust setup

If you are using the vscode rust plugin, here is an example of
the settings I am using

The rust.rls is the most important one.

``` json
// Place your settings in this file to overwrite the default settings
{
    "rust.rls": {
        "executable": "rustup",
        "args": [
            "run",
            "nightly",
            "rls"
        ]
    },
    //"editor.formatOnSave": true,
    //"editor.fontFamily": "Fira Code"
}
```

### Running

build:

`cargo build` or `cargo build --release`

run:

`./target/debug/wssps`, or if built with `--release`, `./target/release/wssps`

use `-p` or `--port` to set http port (default is 8080)

then browse to `http://localhost:PORT` to find the test page.

### Logging

Wssps makes extensive use of logging and the [env_logger](https://crates.io/crates/env_logger) crate
 
Please read the env_logger docs for more information, but here is a quick example:

```RUST_LOG=debug ./target/debug/wssps```

## TODO

* [ ] Break this out into bugs/features :)
* [ ] TLS Support
* [ ] Determine settings to help shrink file size
* [ ] Add command to reset entire serial port managment subsystem
if it looks like things are wedged
* [x] Switch to dynamic timing loops for all msg handling threads
    * [ ] Allow users to specify desired update frequency
    * [x] Log if time per loop is exceeded
* [x] Configuration file support
    * [x] Use [toml](https://github.com/toml-lang/toml)
    * [ ] serial port whitelist/blacklist/regex
    * [x] Specify ip address to bind to besides local host
* [ ] Add HTTPS/WSS support
    * [ ] Specify cert locations
* [ ] Add method to reinitialize serial port subsystem if things
totally go south
* [x] Remove sub_id from SerialRequest and send it as tuple
with sub_id to handler method
* [ ] Reduce the usage of String in favor of &str?
* [ ] "Wrote" response message, should we return a hash of the data that was written so integrity can be verified?
