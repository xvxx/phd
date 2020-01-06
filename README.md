<!--
      /       |
 ___ (___  ___|
|   )|   )|   )
|__/ |  / |__/
|
--> <p align="center"> <img src="./img/logo.png"> <br> 
<a href="https://github.com/dvkt/phd/releases">
<img src="https://img.shields.io/github/v/release/dvkt/phd?include_prereleases">
</a>
</p>

`phd` is an esoteric gopher server for small gopherholes.

point it at a directory and it'll serve up all its text files,
sub-directories, and binary files over gopher. any `.gph` files will
be served up as [gopermaps][map] and executable `.gph` files will be
run as a script with their output served to the client, like cgi!

### special files:

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

### dynamic content:

any `.gph` file that is marked **executable** with be run as if it
were a shell script and its output will be sent to the client. it will
be passed three arguments: the query string (if any, the host, and the
port. do with them what you will.

for example:

```sh
$ cat echo.gph
#!/bin/sh
echo "Hi, world! You said:" $1
echo "1Visit Gopherpedia	/	gopherpedia.com	70"
```

then:

    $ gopher-client gopher://localhost/1/echo?something
    [INFO] Hi, world! You said: something
    [LINK] Visit Gopherpedia

or more seriously:

```sh
$ cat figlet.gph
#!/bin/sh
figlet $1
```

then:

    $ gopher-client gopher://localhost/1/figlet?hi gopher
    [INFO]  _     _                     _
    [INFO] | |__ (_)   __ _  ___  _ __ | |__   ___ _ __
    [INFO] | '_ \| |  / _` |/ _ \| '_ \| '_ \ / _ \ '__|
    [INFO] | | | | | | (_| | (_) | |_) | | | |  __/ |
    [INFO] |_| |_|_|  \__, |\___/| .__/|_| |_|\___|_|
    [INFO]             |___/      |_|

### ruby on rails:

`sh` is fun, but for serious work you need a serious scripting
language like Ruby or PHP or Node.JS:

```ruby
$ cat sizes.gph
#!/usr/bin/env ruby

def filesize(file)
    (size=File.size file) > (k=1024) ? "#{size/k}K" : "#{size}B"
end

puts "~ file sizes ~"
spaces = 20
Dir[__dir__ + "/*"].each do |entry|
    name = File.basename entry
    puts "#{name}#{' ' * (spaces - name.length)}#{filesize entry}"
end
```

now you can finally share the file sizes of a directory with the world
of Gopher! 

    $ phetch -r 0.0.0.0:7070/1/sizes
    i~ file sizes ~	(null)	127.0.0.1	7070
    iCargo.toml          731B	(null)	127.0.0.1	7070
    iLICENSE             1K	(null)	127.0.0.1	7070
    iMakefile            724B	(null)	127.0.0.1	7070
    itarget              288B	(null)	127.0.0.1	7070
    iphd                 248K	(null)	127.0.0.1	7070
    iCargo.lock          2K	(null)	127.0.0.1	7070
    iREADME.md           4K	(null)	127.0.0.1	7070
    img                 96B	(null)	127.0.0.1	7070
    isizes.gph           276B	(null)	127.0.0.1	7070
    isrc                 224B	(null)	127.0.0.1	7070

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

- [phd-v0.1.5-linux-x86_64.tar.gz][0]
- [phd-v0.1.5-linux-armv7.tar.gz (RPi)][1]
- [phd-v0.1.5-macos.zip][2]

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

[0]: https://github.com/dvkt/phd/releases/download/v0.1.5/phd-v0.1.5-linux-x86_64.tar.gz
[1]: https://github.com/dvkt/phd/releases/download/v0.1.5/phd-v0.1.5-linux-armv7.tar.gz
[2]: https://github.com/dvkt/phd/releases/download/v0.1.5/phd-v0.1.5-macos.zip
[map]: https://en.wikipedia.org/wiki/Gopher_(protocol)#Source_code_of_a_menu
