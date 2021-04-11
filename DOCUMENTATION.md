# Documentation

## Configuration

wssps supports configuration in several different ways, by commandline parameters, env variables, and config files

commandline options override env variables, which overrides file based config.

Currently the following values may be specified

* `port` The HTTP /WS port to bind to, defaults to 10080
* `address` The ip address the server binds to, defaults to 127.0.0.1 ( localhost )

When wssps starts, it first tries to load configuration information from the following files: 

TODO UPDATE

**Only the first file found is loaded.**

The configuration file makes use of [TOML](https://github.com/toml-lang/toml). Here is a sample config:

``` toml
# Sample config. Hashes mark comments

port = 10090
address = "10.1.101.26"
```

Next, it tries to pull in config from the environment. These values will override any values found in any loaded configuration files.

The following env variable names are searched:

* `WSSPS_PORT` Specifies the HTTP/WS port
* `WSSPS_ADDRESS` Specifies the ip address to bind to

Finally it parses and uses any configuration passed in via commandline arguments

Available commandline arguments can be found via running `wssps -h` or `wssps --help`

TODO UPDATE

Finally, any item not specified in any of these steps is given the default value mentioned at the beginning of this section.

## Source Docs
For now, run `cargo doc --no-deps` and browse to `target/docs` for 
html based documents