# procs

**procs** is a replacement for `ps` written by [Rust](https://www.rust-lang.org/).

[![Build Status](https://travis-ci.org/dalance/procs.svg?branch=master)](https://travis-ci.org/dalance/procs)
[![Crates.io](https://img.shields.io/crates/v/procs.svg)](https://crates.io/crates/procs)
[![codecov](https://codecov.io/gh/dalance/procs/branch/master/graph/badge.svg)](https://codecov.io/gh/dalance/procs)

## Features

- Output by the colored and human-readable format
- Keyword search over multi-column
- Some additional information (ex. TCP/UDP port, Read/Write throughput) which are not suportted by `ps`

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

```
> procs
```

If you add any keyword as argument, it is matched to `USER` or `Command` by default.

```
> procs dalance  // show the processes of user dalance
```

If an integer number is used as the keyword, it is matched to `PID`, `TCP`, `UDP` by default.

```
> procs 6000  // show the process of pid=6000 or TCP/UDP port=6000
```

### Configuration

This is not implemented yet.
