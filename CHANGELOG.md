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
