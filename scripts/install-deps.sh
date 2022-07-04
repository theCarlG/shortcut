#!/bin/bash -eux

export KEYRINGS=/usr/local/share/keyrings
source /etc/os-release

apt-get update

update-alternatives --remove-all clang
update-alternatives --remove-all clang++

apt-get install --no-install-recommends -y tar cmake python3 gpg curl libpulse-dev libxcb-render0 libxcb-render0 libxcb-xinerama0 libgtk-3-dev \
                                           libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev ca-certificates;
update-ca-certificates

# mold linker
mold_version="1.3.0";
mold_prefix="mold-$mold_version-x86_64-linux";
curl --fail -L https://github.com/rui314/mold/releases/download/v$mold_version/$mold_prefix.tar.gz | tar -xzv -C /usr/local/ --strip-components=1;
mold -v;

mkdir -p $KEYRINGS;


# clang/lld/llvm
curl --fail https://apt.llvm.org/llvm-snapshot.gpg.key | gpg --dearmor > $KEYRINGS/llvm.gpg;
echo "deb [signed-by=$KEYRINGS/llvm.gpg] http://apt.llvm.org/$VERSION_CODENAME/ llvm-toolchain-$VERSION_CODENAME-14 main" > /etc/apt/sources.list.d/llvm.list;
dpkg --add-architecture i386;
apt-get update && apt-get install --no-install-recommends -y clang-14 protobuf-compiler llvm-14 lld-14

ln -s clang-14 /usr/bin/clang
ln -s clang /usr/bin/clang++
ln -s lld-14 /usr/bin/ld.lld
ln -s clang-14 /usr/bin/clang-cl
ln -s llvm-ar-14 /usr/bin/llvm-lib
ln -s lld-link-14 /usr/bin/lld-link

# Verify the symlinks are correct
protoc --version;
clang++ -v;
ld.lld -v;

# Doesn't have an actual -v/--version flag, but it still exits with 0
llvm-lib -v;
clang-cl -v;
lld-link --version;

# Use clang instead of gcc when compiling binaries targeting the host (eg proc macros, build files)
update-alternatives --install /usr/bin/cc cc /usr/bin/clang 100;
update-alternatives --install /usr/bin/c++ c++ /usr/bin/clang++ 100;
