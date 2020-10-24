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

## Using in your CI pipeline

Let's say, normally your CI pipeline would do something like.

```bash
docker build -f someproject/Dockerfile .
```

Some problems with this method are:

* It takes some time for all the files of this build to be sent to docker deamon.
  This part alone can can take a substantial time, even in the happy case that nothing
  needs rebuilding since the container image is already cached locally.
* If exactly the same build was already done on some different machine, it will
  not be reused on this one, unless you have some smarter system set up to share them.
* You need to wait for the `docker build` to complete to get a unique id of the build.

With DSC you would:

```bash
BUILD_FULL_ID=$(docker-source-checksum -f someproject/Dockerfile .)
BUILD_ID=${BUILD_FULL_ID:0:8} # take just first 8 characters
TAG_NAME=my-docker-repository.com/$PACKAGE_NAME:$BUILD_ID
```

and in less than a second, even for a big project, you get a deterministic cryptographic ID
of the build *without attemting to build anything just yet* .
At this point, you can potentially speculatively start parts of your CI
with an already known docker image url.

Rest of your CI script can quickly check if this exact build already exists with:

```bash
if DOCKER_CLI_EXPERIMENTAL=enabled docker manifest inspect $TAG_NAME > /dev/null; then
  echo "$TAG_NAME already built. Skipping build and push"
  exit 0
fi
```

(or just `docker pull` if you want it cached locally too).

And only if it was not ever built, only then you build locally and push it to your registry:

```bash
docker build -t $TAG_NAME -f someproject/Dockerfile .
docker push $TAG_NAME
```


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
