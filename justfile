# This is powered by `just`, see <https://github.com/casey/just>,
# <https://just.systems/man/en/>
default:
    echo 'Hello, world!'

git fetch:
    git fetch --all --recurse-submodules

cargo check extendrtests:
    cargo check --manifest-path tests/extendrtests/src/rust/Cargo.toml

cargo expand extendrtests:
    cargo expand --manifest-path tests/extendrtests/src/rust/Cargo.toml

# Initialises and updates all submodules
# Installs all packages needed to run `cargo extendr devtools-tests`
configure:
    just git fetch
    git submodule update --init
    Rscript -e "options(repos = c(CRAN = 'http://cran.rstudio.com'))" \
    -e "install.packages('attachment');" \
    -e "attachment::install_from_description('tests/extendrtests/DESCRIPTION')"
