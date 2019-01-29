# procs

**procs** is a replacement for `ps` written by [Rust](https://www.rust-lang.org/).

[![Build Status](https://travis-ci.org/dalance/procs.svg?branch=master)](https://travis-ci.org/dalance/procs)
[![Crates.io](https://img.shields.io/crates/v/procs.svg)](https://crates.io/crates/procs)
[![codecov](https://codecov.io/gh/dalance/procs/branch/master/graph/badge.svg)](https://codecov.io/gh/dalance/procs)

## Features

- Output by the colored and human-readable format
- Keyword search over multi-column
- Some additional information (ex. TCP/UDP port, Read/Write throughput) which are not supported by `ps`

## Platform

Linux is supported only.

## Installation

### Download binary

Download from [release page](https://github.com/dalance/procs/releases/latest), and extract to the directory in PATH.

### Cargo

You can install by [cargo](https://crates.io).

```
cargo install procs
```

## Usage

Type `procs` only. It shows the information of all processes.

```console
$ procs
```

![procs](https://user-images.githubusercontent.com/4331004/51904370-d5ad9180-2401-11e9-837c-ae4859c8fa82.png)

If you add any keyword as argument, it is matched to `USER` or `Command` by default.
( `--mask` option is used to mask `USER`/`Command` information, actually not required )

```console
$ procs zsh --mask
```

![procs_zsh](https://user-images.githubusercontent.com/4331004/51904402-e827cb00-2401-11e9-8a9c-45159686080d.png)

If an integer number is used as the keyword, it is matched to `PID`, `TCP`, `UDP` by default.
Integer is treated as exact match, and other keyword is treated as partial match.

```console
$ procs 6000 60000 60001 --mask
```

![procs_port](https://user-images.githubusercontent.com/4331004/51904423-f5dd5080-2401-11e9-8d02-756e33a9b7bc.png)

### Configuration

This is not implemented yet.
