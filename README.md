# Gopher server in Rust

## Running

```
$ rust_gopher [-h hostname] [-p port] [-d dir]
```

hostname - hostname to include it links to local files (defaults to localhost)
port - to listen on (defaults to 70)
dir - where to find content (defaults to dir called `root` in cwd)