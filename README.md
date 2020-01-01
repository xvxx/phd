<!--
      /       |
 ___ (___  ___|
|   )|   )|   )
|__/ |  / |__/
|
--> <p align="center"><img src="./img/logo.png"></p>

`phd` is an esoteric gopher server for small gopherholes.

point it at a directory and it'll serve up all its text files,
sub-directories, and binary files over gopher. any `.gph` files will
be served up as [gopermaps][map] and executable `.gph` files will be
run as a script with their output served to the client, like cgi!

special files:

- **header.gph**: if it exists in a directory, its content will be
  shown above the directory's content. put ascii art in it.
- **footer.gph**: same, but will be shown below a directory's content.
- **index.gph**: completely replaces a directory's content with what's
  in this file.
- **??.gph**: visiting gopher://yoursite/1/dog/ will try to render
  `dog.gph` from disk.
- **.reverse**: if this exists, the directory contents will be listed
  in reverse alphanumeric order. useful for phloggin'.

any line in a `.gph` file that doesn't contain tabs (`\t`) and doesn't
start with an `i` will get an `i` automatically prefixed, turning it
into a gopher information item.

any `.gph` file that is marked **executable** with be run as if it
were a shell script and its output will be sent to the client. it will
be passed three arguments: the query string (if any, the host, and the
port. do with them what you will.

for example:

    $ cat echo.gph
    #!/bin/sh
    echo "Hi, world! You said:" $1
    echo "1Visit Gopherpedia	/	gopherpedia.com	70"

then:

    $ gopher-client gopher://localhost/1/echo?something
    [INFO] Hi, world! You said: something
    [LINK] Visit Gopherpedia

or more seriously:

    $ cat figlet.gph
    #!/bin/sh
    figlet $1

then:

    $ gopher-client gopher://localhost/1/figlet?hi gopher
    [INFO]  _     _                     _
    [INFO] | |__ (_)   __ _  ___  _ __ | |__   ___ _ __
    [INFO] | '_ \| |  / _` |/ _ \| '_ \| '_ \ / _ \ '__|
    [INFO] | | | | | | (_| | (_) | |_) | | | |  __/ |
    [INFO] |_| |_|_|  \__, |\___/| .__/|_| |_|\___|_|
    [INFO]             |___/      |_|


## usage

    Usage:

        phd [options] <root directory>

    Options:

        -p, --port      Port to bind to.
        -h, --host      Hostname to use when generating links.

    Other flags:

        -h, --help      Print this screen.
        -v, --version   Print phd version.

    Examples:

        phd ./path/to/site  # Serve directory over port 7070.
        phd -p 70 docs      # Serve 'docs' directory on port 70
        phd -h gopher.com   # Serve current directory over port 7070
                            # using hostname "gopher.com"

## installation

binaries for linux, mac, and raspberry pi are available
at https://github.com/dvkt/phd/releases:

- [phd-v0.1.4-x86_64.tar.gz][0]
- [phd-v0.1.4-armv7.tar.gz (RPi)][1]
- [phd-v0.1.4-macos.zip][2]

just unzip/untar the `phd` program into your $PATH and get going!

## development

    cargo run -- ./path/to/gopher/site

## resources

- https://github.com/gophernicus/gophernicus/blob/master/README.Gophermap
- https://gopher.zone/posts/how-to-gophermap/
- [rfc 1436](https://tools.ietf.org/html/rfc1436)

## todo

- [ ] script/serverless mode
- [ ] systemd config, or something
- [ ] TLS support
- [ ] man page
- [ ] ipv6

[0]: https://github.com/dvkt/phd/releases/download/v0.1.3/phd-v0.1.4-x86_64.tar.gz
[1]: https://github.com/dvkt/phd/releases/download/v0.1.3/phd-v0.1.4-armv7.tar.gz
[2]: https://github.com/dvkt/phd/releases/download/v0.1.3/phd-v0.1.4-macos.zip
[map]: https://en.wikipedia.org/wiki/Gopher_(protocol)#Source_code_of_a_menu
