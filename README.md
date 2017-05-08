# websocket-serial-server
Connect to and read / write serial ports over websockets. In Rust

**VERY VERY VERY ALPHA**

**DOES NOT WORK YET**

## Dependencies

### Linux

1. `sudo apt-get install libudev-dev`
1. `sudo apt-get install libssl-dev`
1. `sudo apt-get install pkg-config`

## TODO
* [ ] Determine settings to help shrink file size
* [ ] Add command to reset entire serial port managment subsystem
if it looks like things are wedged
 
## Limitations

Currently Websocket-rs is not tokio based, so it spawns a thread per connection.
For having a few clients talk to a 3D printer, CNC machine, or other 
such use case, this is normally fine. 

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
    "editor.formatOnSave": true,
    "editor.fontFamily": "Fira Code"
}
```
