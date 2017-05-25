# Documentation

## Configuration

wsss supports configuration in several different ways, by commandline parameters, env variables, and config files

commandline options override env variables, which overrides file based config.

Currently the following values may be specified

* `http_port` The HTTP port to bind to, defaults to 10080
* `ws_port` The port the websocket listens on, defaults to 10081
* `bind_address` The ip address the server binds to, defaults to 127.0.0.1 ( localhost )

When wsss starts, it first tries to load configuration information from the following files: 

1. The file specified by the environment variable `WSS_CONF_FILE`
1. It then tries to load the file in `/etc/wsss/wsss_conf.toml`
1. It then tries to load a `wsss_conf.toml` file located in the same directory as the wsss executable

**Only the first file found is loaded.**

The configuration file makes use of [TOML](https://github.com/toml-lang/toml). Here is a sample config:

``` toml
# Sample config. Hashes mark comments

http_port = 10090
ws_port = 10095
bind_address = "10.1.101.26"
```

Next, it tries to pull in config from the environment. These values will override any values found in any loaded configuration files.

The following env variable names are searched:

* `WSSS_HTTP_PORT` Specifies the HTTP port
* `WSSS_WS_PORT` Specifies the Websocket port
* `WSSS_BIND_ADDRESS` Specifies the ip address to bind to

Finally it parses and uses any configuration passed in via commandline arguments

Available commandline arguments can be found via running `wsss -h` or `wsss --help`

sample output:

```
Usage:
    ./target/debug/wsss [OPTIONS]

Provide access to serial ports over JSON Websockets

optional arguments:
  -h,--help             show this help message and exit
  -p,--http_port HTTP_PORT
                        Http Port
  -w,--ws_port WS_PORT  Websocket Port
  -a,--bind_address BIND_ADDRESS
                        Bind Address
```

Finally, any item not specified in any of these steps is given the default value mentioned at the beginning of this section.

## Source Docs
For now, run `cargo doc --no-deps` and browse to `target/docs` for 
html based documents