# websocket-serial-server
Connect to and read / write serial ports over websockets. In Rust

## Dependencies

### Linux

1. `sudo apt-get install libudev-dev`
1. `sudo apt-get install libssl-dev`
1. `sudo apt-get install pkg-config`

## TODO
* [ ] Determine settings to help shrink file size
 
## Limitations

Currently Websocket-rs is not tokio based, so it spawns a thread per connection.
For having a few clients talk to a 3D printer, CNC machine, or other 
such use case, this is normally fine. 

Once Websocket-rs moves to tokio, this limitation can be removed
