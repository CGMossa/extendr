# This is powered by `just`, see <https://github.com/casey/just>,
# <https://just.systems/man/en/>
default:
    echo 'Hello, world!'

git fetch:
    git fetch --all --recurse-submodules

cargo check extendrtests:
    cargo check --manifest-path tests/extendrtests/src/rust/Cargo.toml

configure:
    git submodule update --init

