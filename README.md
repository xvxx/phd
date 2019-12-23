```
      /       |
 ___ (___  ___|
|   )|   )|   )
|__/ |  / |__/
|
```

dirt simple gopher server.

## todo

- [ ] serve directory listing
- [ ] serve text file
- [ ] serve binary (mp3, exe)
- [ ] index.gophermap
- [ ] footer.gophermap
- [ ] header.gophermap

## usage

    phd [options] <directory>

    phd ./path/to/gopher/root    # Serve directory over port 70.
    phd -p 7070 docs             # Serve 'docs' directory on port 7070
    phd -h localhost             # Serve cwd using hostname "localhost".

## development

    cargo run -- ./path/to/gopher/site

## resources

- https://github.com/gophernicus/gophernicus/blob/master/README.Gophermap
- https://gopher.zone/posts/how-to-gophermap/
- [rfc 1346](https://tools.ietf.org/html/rfc1436)

