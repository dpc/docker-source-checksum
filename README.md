# Deterministic source-based docker image checksum

# Use case

You have a CI pipeline that builds a monorepo with many Dockerfiles.

You want to efficiently avoid rebuilding Dockerfiles that haven't changed,
even when the rest of the monorepo did.

`docker-source-checksum` will calculate a hash of:

* `Dockerfile` content
* all source files referenced by that `Dockerfile` (by parsing it)
* any additiona arguments that might affect the build

and then hashing all of these together, to give you deterministic checksum,
before you even attempt to call `docker build`. You can use it as a
deterministic content-based ID to avoid rebuilding containers that
were already built (eg. by taging them with that checksum).

# Warnings and missing features

* don't use it on untrusted `Dockerfiles`
* the exact checksum is not stable yet and can change between versions
* `["src1", "src", "dst"]` syntax of `ADD` and `COPY` is not supported (PRs welcome)
* file modes and ownership is ignored
* it was put together in 2 hours, so if you plan to use it in production, maybe... review the code or something and tell me it's OK

Having said that, seems to work great.
