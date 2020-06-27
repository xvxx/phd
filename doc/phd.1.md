PHD(1)

# NAME

phd - an estoeric gopher server

# SYNOPSIS

*phd* [_OPTIONS_] [_SITE ROOT_]

# DESCRIPTION

*phd* is a small, easy-to-use gopher server.

Point it at a directory and it'll serve up all the text files,
sub-directories, and binary files over Gopher. Any *.gph* files will
be served up as Gophermaps and executable *.gph* files will be
run as a program with their output served to the client, like the
glorious cgi-bin days of yore!

Usually *phd* is started with a path to your Gopher site:

	phd /srv/gopher

If no path is given, *phd* will use the current directory as the root
of your Gopher site.

# OPTIONS

*-r* _SELECTOR_, *--render* _SELECTOR_

	Rather than start as a server, render the _SELECTOR_ of the site using the options provided and print the raw response to *STDOUT*.

*-b* _ADDRESS_, *--bind* _ADDRESS_
	Set the socket address to bind to, e.g. *127.0.0.1:7070*

*-p* _PORT_, *--port* _PORT_
	Set the _PORT_ to use when generating Gopher links.

*-h* _HOST_, *--host* _HOST_
	Set the _HOST_ to use when generating Gopher links.

*-h*, *--help*
	Print a help summary and exit.

*-v*, *--version*
	Print version information and exit.

# SPECIAL FILES

The following files have special behavior when present in a directory
that *phd* is tasked with serving:

*header.gph*
	If it exists in a directory, its content will be shown above the directory's content. Put ASCII art in it.

*footer.gph*
	Same, but will be shown below a directory's content.

*index.gph*
	Completely replaces a directory's content with what's in this file.

*??.gph*
	Visiting *gopher://yoursite/1/dog/* will try to render *dog.gph* from disk. Visiting */1/dog.gph* will render the raw content of the .gph file.

*.reverse*
	If this exists, the directory contents will be listed in reverse alphanumeric order. Useful for phloggin', if you date your posts.

# GOPHERMAP SYNTAX

Any line in a *.gph* file that doesn't contain tabs (*\t*) will get an
*i* automatically prefixed, turning it into a Gopher information item.

For your convenience, phd supports *geomyidae* syntax for
creating links:

```
This is an info line.
[1|This is a link|/help|server|port]
[h|URL Link|URL:https://noogle.com]
```

*server* and *port* will get translated into the server and port of
the actively running server, eg *localhost* and *7070*.

Any line containing a tab character (*\t*) will be sent as-is to the
client, meaning you can write and serve up raw Gophermap files too.

# DYNAMIC CONTENT

Any *.gph* file that is marked *executable* with be run as if it
were a standalone program and its output will be sent to the client.
It will be passed three arguments: the query string (if any), the
server's hostname, and the current port. Do with them what you will.

For example:

```
$ cat echo.gph
#!/bin/sh
echo "Hi, world! You said:" $1
echo "1Visit Gopherpedia	/	gopherpedia.com	70"
```

Then:

```
$ gopher-client gopher://localhost/1/echo?something
[INFO] Hi, world! You said: something
[LINK] Visit Gopherpedia
```

Or more seriously:

```
$ cat figlet.gph
#!/bin/sh
figlet $1
```

then:

```
$ gopher-client gopher://localhost/1/figlet?hi gopher
[INFO]  _	 _					 _
[INFO] | |__ (_)   __ _  ___  _ __ | |__   ___ _ __
[INFO] | '_ \| |  / _` |/ _ \| '_ \| '_ \ / _ \ '__|
[INFO] | | | | | | (_| | (_) | |_) | | | |  __/ |
[INFO] |_| |_|_|  \__, |\___/| .__/|_| |_|\___|_|
[INFO]			|___/	  |_|
```

## RESOURCES

geomyidae source code
	gopher://bitreich.org/1/scm/geomyidae/files.gph

Example Gophermap
	https://github.com/gophernicus/gophernicus/blob/master/README.Gophermap

Gophermaps
	https://gopher.zone/posts/how-to-gophermap/

RFC 1436:
	https://tools.ietf.org/html/rfc1436

# ABOUT

*phd* is maintained by chris west and released under the MIT license.

phd's Gopher hole:
	_gopher://phkt.io/1/phd_
phd's webpage:
	_https://github.com/xvxx/phd_
