# Deterministic source-based docker image checksum

## Use case

You have a CI pipeline that builds a monorepo with many Dockerfiles.

You want to efficiently avoid rebuilding Dockerfiles that haven't changed,
even when the rest of the monorepo did.

`docker-source-checksum` will calculate a hash of:

* `Dockerfile` content
* all source files referenced by that `Dockerfile` (figured out by parsing it)
* any additiona arguments that might affect the build

and then hashing all of these together, to give you deterministic checksum,
before you even attempt to call `docker build`. You can use it as a
deterministic content-based ID to avoid rebuilding containers that
were already built (eg. by taging them with that checksum).

## Warnings and missing features

* don't use it on untrusted `Dockerfiles`
* the exact checksum is not stable yet and can change between versions
* `["src1", "src", "dst"]` syntax of `ADD` and `COPY` is not supported (PRs welcome)
* file ownership is ignored
* it was put together in 2 hours, so if you plan to use it in production, maybe... review the code or something and tell me what you think

Having said that, seems to work great.

## Installing

See [docker-source-checksum releases](https://github.com/dpc/docker-source-checksum/releases),
or use `cargo install docker-source-checksum`.

## Using

Somewhat similiar to `docker build`:

```
$ docker-source-checksum --help
docker-source-checksum 0.2.0
Dockerfile source checksum

USAGE:
    docker-source-checksum [FLAGS] [OPTIONS] <context-path>

FLAGS:
    -h, --help       Prints help information
        --hex        Output hash in hex
    -V, --version    Prints version information

OPTIONS:
        --extra-path <extra-path>...        Path relative to context to include in the checksum
        --extra-string <extra-string>...    String (like arguments to dockerfile) to include in the checksum
    -f, --file <file>                       Path to `Dockerfile`
        --ignore-path <ignore-path>...      Path relative to context to ignore in the checksum

ARGS:
    <context-path>    Dockerfile build context path
```
