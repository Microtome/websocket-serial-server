# websocket-serial-server
Connect to and read / write serial ports over websockets. In Rust

**VERY VERY VERY ALPHA**

## Dependencies

### Linux

1. `sudo apt-get install libudev-dev`
1. `sudo apt-get install libssl-dev`
1. `sudo apt-get install pkg-config`

### Windows

Unknown, help appreciated.

### OSX

Unknown, help appreciated.

## TODO

* [ ] Determine settings to help shrink file size
* [ ] Add command to reset entire serial port managment subsystem
if it looks like things are wedged
* [ ] Switch to dynamic timing loops for all msg handling threads
    * [ ] Allow users to specify desired update frequency
    * [ ] Log if time per loop is exceeded
* [ ] Configuration file support
    * [ ] Use [toml](https://github.com/toml-lang/toml)
    * [ ] serial port whitelist/blacklist/regex
    * [ ] Specify ip address to bind to besides local host
* [ ] Add HTTPS/WSS support
    * [ ] Specify cert locations
* [ ] Add method to reinitialize serial port subsystem if things
totally go south
* [x] Remove sub_id from SerialRequest and send it as tuple
with sub_id to handler method
* [ ] Use something like Rustache to manage the html template? That
way brackets don't need doubling up.
* [ ] Reduce the usage of String in favor of &str?


## Limitations

Currently Websocket-rs is not tokio based, so it spawns a thread per connection.
For having a few clients talk to a 3D printer, CNC machine, or other 
such use case, this is fine. 

Once Websocket-rs moves to tokio, this limitation can be removed

## Vscode-Rust setup

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

