#!/bin/bash

set -x

if [ -z "$TRAVIS_TAG" ]; then
    name="$PROJECT_NAME-$TRAVIS_BRANCH-$TARGET"
else
    name="$PROJECT_NAME-$TRAVIS_TAG-$TARGET"
fi

mkdir -p "$name"
cp "target/$TARGET/release/$PROJECT_NAME" "$name/" \
	|| cp "target/$TARGET/release/$PROJECT_NAME.exe" "$name/"
cp README.md LICENSE* "$name/"
tar czvf "$name.tar.gz" "$name"

# FIXME: Windows does come with `shasum`?
if [[ "$TARGET" =~ "-windows-" ]]; then
    echo "FIXME: shasum not available on windows targets"
else
    # Get the sha-256 checksum w/o filename and newline
    echo -n "$(shasum -ba 256 "$name.tar.gz" | cut -d " " -f 1)" > "$name.tar.gz.sha256"
fi

