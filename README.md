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
