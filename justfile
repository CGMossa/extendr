# This is powered by `just`, see <https://github.com/casey/just>,
# <https://just.systems/man/en/>
default:
    echo 'Hello, world!'

git fetch:
    git fetch --all --recurse-submodules

configure:
    git submodule update --init

