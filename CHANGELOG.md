## v0.1.12-dev

`phd` now uses `-b` and `--bind` to set the host and port to
bind to. `-p` and `-h` are now strictly for URL generation.

This should hopefully make it easier to run `phd` behind a
proxy and still generate proper links.

Thanks to @bradfier for the patch!

## v0.1.11

`phd` now ships with a basic manual!

It will be installed via homebrew and (eventually) AUR.

For now you can view it by cloning the repository and running:

    man ./doc/phd.1

Enjoy!


## v0.1.10

`phd` can now render a single page to stdout, instead of starting
as a server. Those of us in the biz refer to this as "serverless".

For example, if your Gopher site lives in `/srv/gopher` and you want
to render the main page, just run:

    phd -r / /srv/gopher

This will print the raw Gopher menu to stdout!

To view the "/about" page, pass that selector:

    phd -r / /srv/gopher

Edge computing is now Gopher-scale! Enjoy!

## v0.1.9

Switch to using GitHub Actions for release automation.
