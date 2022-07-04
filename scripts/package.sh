#!/bin/bash -eux


PKG_VERSION=${PKG_VERSION:-dev}
ROOT=$(realpath $(dirname "${BASH_SOURCE[0]}")/../)
STAGING=shortcut-$PKG_VERSION
TMP=$(mktemp -d)
DEST=$TMP/$STAGING

test -d $DEST && ( echo "Removing old dir"; rm -vrf $DEST )
echo -e "\nCreating dirs"
mkdir -vp $DEST/images

echo -e "\nCopying files"
for bin in $ROOT/target/x86_64-unknown-linux-gnu/release/shortcut-gui $ROOT/target/x86_64-unknown-linux-musl/release/shortcut-daemon; do
    if [ -f $bin ]; then
        cp -v $bin "$DEST/$(basename $bin)"
    fi
done

cp -v $ROOT/shortcut-gui/resources/art/*.png $DEST/images/

cp -v {README.md,COPYING,LICENSE} "$DEST/"
{
    cd $TMP

    echo -e "\nCreate archive ${STAGING}.tar.gz"
    tar -vczf $ROOT/${STAGING}.tar.gz "$STAGING"

    echo -e "\nRemoving $DEST"
}
rm -vrf $TMP
