```
      /       |
 ___ (___  ___|
|   )|   )|   )
|__/ |  / |__/
|
```

esoteric gopher server. 

## todo

- [ ] index.gph
- [ ] footer.gph
- [ ] header.gph
- [ ] *.gph

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
- [rfc 1436](https://tools.ietf.org/html/rfc1436)

