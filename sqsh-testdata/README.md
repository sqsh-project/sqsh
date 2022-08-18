# Sqsh Testdata Generator

A tool to generate synthetic floating point datasets.
Currently the only datasets with a [gaussian/normal](https://en.wikipedia.org/wiki/Normal_distribution) distribution are supported.
Further distributions can be added if the need may rise.

## Install

Use stable `cargo` release to install the toolset:

```sh
cargo install sqsh-testdata
```

Use latest version from github:

```sh
git clone https://github.com/sqsh-project/testdata.git && \
cd sqsh-testdata && \
cargo install --path .
```

## Usage

The easiest way to generate a dataset is to specify `mean`, `standard deviation` and the number of numbers to be generated via `num`:

```sh
sqsh-testdata --mean 10 --std 2 --size 1000 # mandatory arguments
```

The default generates single-precision floating-point data. This can be changed using the `--datatype` argument:

```sh
sqsh-testdata --mean 10 --std 2 --size 1000 --datatype double
```

Should it be necessary to define a different endianess than the machine native the `--endianess` argument can be used

```sh
sqsh-testdata --mean 10 --std 2 --size 1000 --endianess little
```

By providing a `--seed` value the generated data can be reproduced on different environments:

```sh
sqsh-testdata --mean 10 --std 2 --size 1000 --seed 42
```

The data can be output to a file by piping the result to a file:

```sh
sqsh-testdata --mean 10 --std 2 --size 1000 > /tmp/data.raw  # save to file
```

For more information the help menu can be used: `td --help`
