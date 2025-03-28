# Change Log

## [Unreleased](https://github.com/dalance/procs/compare/v0.14.10...Unreleased) - ReleaseDate

## [v0.14.10](https://github.com/dalance/procs/compare/v0.14.9...v0.14.10) - 2025-03-28

* [Added] Add Arch column for macOS (Rosetta 2) [#759](https://github.com/dalance/procs/pull/759)
* [Added] Add JSON output support [#735](https://github.com/dalance/procs/pull/735)
* [Changed] Generate shell completions at build time [#739](https://github.com/dalance/procs/pull/739)
* [Fixed] Trim null byte from /proc/<pid>/attr/current [#733](https://github.com/dalance/procs/pull/733)

## [v0.14.9](https://github.com/dalance/procs/compare/v0.14.8...v0.14.9) - 2025-01-17

* [Fixed] Build failure on FreeBSD

## [v0.14.8](https://github.com/dalance/procs/compare/v0.14.7...v0.14.8) - 2024-10-23

* [Fixed] Build breaks on FreeBSD [#703](https://github.com/dalance/procs/issues/703)

## [v0.14.7](https://github.com/dalance/procs/compare/v0.14.6...v0.14.7) - 2024-10-22

* [Changed] Header line wrapping [#695](https://github.com/dalance/procs/pull/695)
* [Fixed] First key press is ignored [#443](https://github.com/dalance/procs/issues/443)

## [v0.14.6](https://github.com/dalance/procs/compare/v0.14.5...v0.14.6) - 2024-07-30

* [Changed] MSRV to Rust 1.74
* [Added] aarch64-apple-darwin build release

## [v0.14.5](https://github.com/dalance/procs/compare/v0.14.4...v0.14.5) - 2024-03-07

* [Added] Add show_self_parents option [#607](https://github.com/dalance/procs/pull/607)
* [Changed] MSRV to Rust 1.70

## [v0.14.4](https://github.com/dalance/procs/compare/v0.14.3...v0.14.4) - 2023-11-24

* [Fixed] Build breaks on FreeBSD/arm64,i386

## [v0.14.3](https://github.com/dalance/procs/compare/v0.14.2...v0.14.3) - 2023-10-20

* [Changed] MSRV to Rust 1.67
* [Added] Some columns on FreeBSD

## [v0.14.2](https://github.com/dalance/procs/compare/v0.14.1...v0.14.2) - 2023-10-18

* [Added] BSD support [#313](https://github.com/dalance/procs/issues/313)
* [Fixed] wrong time handling on Windows
* [Added] User/group cache support
* [Added] Cgroup/Ccgroup column [#529](https://github.com/dalance/procs/issues/529)

## [v0.14.1](https://github.com/dalance/procs/compare/v0.14.0...v0.14.1) - 2023-10-06

* [Added] Also look for a config file in /etc/procs/procs.toml [#533](https://github.com/dalance/procs/pull/533)
* [Added] less compatible keybinding of built-in pager
* [Added] `show_kthreads` config [#446](https://github.com/dalance/procs/pull/446)
* [Fixed] procs -i Pid displays Parent PID, not PID, sometimes [#457](https://github.com/dalance/procs/issues/457)

## [v0.14.0](https://github.com/dalance/procs/compare/v0.13.4...v0.14.0) - 2023-03-07

* [Changed] `--config` option to `--gen-config`
* [Changed] `--completion` option to `--gen-completion`
* [Changed] `--completion-out` option to `--gen-completion-out`
* [Added] `--load-config` option to specify config file [#394](https://github.com/dalance/procs/issues/394)
* [Added] `--use-config` option to specify built-in config [#152](https://github.com/dalance/procs/pull/152)
* [Added] `show_header` and `show_footer` config [#405](https://github.com/dalance/procs/issues/405)
* [Added] SecContext column [#260](https://github.com/dalance/procs/issues/260)
* [Added] FileName column [#429](https://github.com/dalance/procs/issues/429)
* [Added] WorkDir column [#410](https://github.com/dalance/procs/issues/410)
* [Added] Env column [#143](https://github.com/dalance/procs/issues/143)
* [Added] Built-in pager and Windows pager support [#119](https://github.com/dalance/procs/issues/119)
* [Fixed] hang on terminals which ignore DSR request [#288](https://github.com/dalance/procs/issues/288)
* [Fixed] Column UserLogin shows 4294967295 [#441](https://github.com/dalance/procs/issues/441)

## [v0.13.4](https://github.com/dalance/procs/compare/v0.13.3...v0.13.4) - 2023-01-29

* [Added] adding sort column to inserts [#396](https://github.com/dalance/procs/pull/396)
* [Added] docker: Respect $DOCKER_HOST [#424](https://github.com/dalance/procs/pull/424)

## [v0.13.3](https://github.com/dalance/procs/compare/v0.13.2...v0.13.3) - 2022-10-18

* [Changed] Release zip for Windows has the exe at toplevel

## [v0.13.2](https://github.com/dalance/procs/compare/v0.13.1...v0.13.2) - 2022-10-05

* [Fixed] invalid charset name issue [#366](https://github.com/dalance/procs/issues/366)

## [v0.13.1](https://github.com/dalance/procs/compare/v0.13.0...v0.13.1) - 2022-09-20

* [Added] session column on macOS [#361](https://github.com/dalance/procs/pull/361)

## [v0.13.0](https://github.com/dalance/procs/compare/v0.12.3...v0.13.0) - 2022-07-29

* [Changed] Update procfs to v0.13.0
* [Changed] Use once_cell instead of lazy_static
* [Added] Case sensitivity option [#159](https://github.com/dalance/procs/issues/159)
* [Added] TreeSlot column [#196](https://github.com/dalance/procs/issues/196)
* [Added] Add TcpPort column support for Windows [#318](https://github.com/dalance/procs/pull/318)
* [Changed] Update dockworker to v0.0.24

## [v0.12.3](https://github.com/dalance/procs/compare/v0.12.2...v0.12.3) - 2022-05-25

* [Fixed] Using bash on Emacs, procs-0.12.2 is very slow compared to procs-0.11.13 [#291](https://github.com/dalance/procs/issues/291)

## [v0.12.2](https://github.com/dalance/procs/compare/v0.12.1...v0.12.2) - 2022-05-05

* [Changed] Update Makefile to change release zip names [#279](https://github.com/dalance/procs/pull/279)

## [v0.12.1](https://github.com/dalance/procs/compare/v0.12.0...v0.12.1) - 2022-01-27

* [Fixed] latency based termbg timeout [#221](https://github.com/dalance/procs/issues/221)
* [Fixed] wrong decode of cgroup for docker [#236](https://github.com/dalance/procs/issues/236)

## [v0.12.0](https://github.com/dalance/procs/compare/v0.11.13...v0.12.0) - 2022-01-18

* [Changed] Update getch to update termios [#223](https://github.com/dalance/procs/issues/223)
* [Changed] Replace structopt with clap
* [Fixed] unexpected message at piped [#221](https://github.com/dalance/procs/issues/221)

## [v0.11.13](https://github.com/dalance/procs/compare/v0.11.12...v0.11.13) - 2021-12-24

* [Added] --completion-out option [#219](https://github.com/dalance/procs/pull/219)

## [v0.11.12](https://github.com/dalance/procs/compare/v0.11.11...v0.11.12) - 2021-12-15

* [Fixed] unexpected debug message

## [v0.11.11](https://github.com/dalance/procs/compare/v0.11.10...v0.11.11) - 2021-12-14

* [Fixed] panic when stdout is closed [#210](https://github.com/dalance/procs/issues/210)

## [v0.11.10](https://github.com/dalance/procs/compare/v0.11.9...v0.11.10) - 2021-10-19

* [Added] pgid/session column [#150](https://github.com/dalance/procs/pull/150)
* [Added] floating point watch interval support [#157](https://github.com/dalance/procs/pull/157)
* [Added] MultiSlot column [#180](https://github.com/dalance/procs/issues/180)
* [Fixed] Search failure with only option [#117](https://github.com/dalance/procs/issues/117)
* [Added] Show children processes at tree mode [#181](https://github.com/dalance/procs/issues/181)

## [v0.11.9](https://github.com/dalance/procs/compare/v0.11.8...v0.11.9) - 2021-06-22

## [v0.11.8](https://github.com/dalance/procs/compare/v0.11.7...v0.11.8) - 2021-05-28

## [v0.11.7](https://github.com/dalance/procs/compare/v0.11.6...v0.11.7) - 2021-05-19

* [Fixed] crash at piped/redirected [#146](https://github.com/dalance/procs/issues/146)
* [Added] elapsed time [#120](https://github.com/dalance/procs/issues/120)
* [Added] completion file message [#130](https://github.com/dalance/procs/issues/130)

## [v0.11.6](https://github.com/dalance/procs/compare/v0.11.5...v0.11.6) - 2021-05-18

* [Fixed] termbg byte leak

## [v0.11.5](https://github.com/dalance/procs/compare/v0.11.4...v0.11.5) - 2021-05-06

* [Fixed] crash at tree mode [#129](https://github.com/dalance/procs/issues/129)

## [v0.11.4](https://github.com/dalance/procs/compare/v0.11.3...v0.11.4) - 2021-03-12

* [Fixed] suppress theme detection at each refresh of watcher mode

## [v0.11.3](https://github.com/dalance/procs/compare/v0.11.2...v0.11.3) - 2021-01-30

* [Changed] default color for light theme

## [v0.11.2](https://github.com/dalance/procs/compare/v0.11.1...v0.11.2) - 2021-01-29

* [Added] obsolete config check

## [v0.11.1](https://github.com/dalance/procs/compare/v0.11.0...v0.11.1) - 2021-01-28

* [Added] thread information [#30](https://github.com/dalance/procs/issues/30)

## [v0.11.0](https://github.com/dalance/procs/compare/v0.10.10...v0.11.0) - 2021-01-28

* [Added] automatic theme detection [#78](https://github.com/dalance/procs/issues/78)

## [v0.10.10](https://github.com/dalance/procs/compare/v0.10.9...v0.10.10) - 2020-11-26

* [Fixed] broken pager on macOS [#92](https://github.com/dalance/procs/issues/92)

## [v0.10.9](https://github.com/dalance/procs/compare/v0.10.8...v0.10.9) - 2020-11-24

* [Added] --completion option [#86](https://github.com/dalance/procs/issues/86)
* [Fixed] crash by --only optiont [#85](https://github.com/dalance/procs/issues/85)

## [v0.10.8](https://github.com/dalance/procs/compare/v0.10.7...v0.10.8) - 2020-11-23

* [Changed] Add `LESSCHARSET=utf-8` by default [#75](https://github.com/dalance/procs/issues/75)

## [v0.10.7](https://github.com/dalance/procs/compare/v0.10.6...v0.10.7) - 2020-11-05

* [Added] detect_width config [#76](https://github.com/dalance/procs/issues/76)

## [v0.10.6](https://github.com/dalance/procs/compare/v0.10.5...v0.10.6) - 2020-11-05

* [Added] --only option [#77](https://github.com/dalance/procs/issues/77)
* [Added] --no-header option [#77](https://github.com/dalance/procs/issues/77)

## [v0.10.5](https://github.com/dalance/procs/compare/v0.10.4...v0.10.5) - 2020-09-26

* [Added] LookupAccountSidW caching [#71](https://github.com/dalance/procs/issues/71)
* [Changed] Move configuration note to help message [#57](https://github.com/dalance/procs/issues/57)

## [v0.10.4](https://github.com/dalance/procs/compare/v0.10.3...v0.10.4) - 2020-08-10

* [Added] 256 colors support [#67](https://github.com/dalance/procs/issues/67)

## [v0.10.3](https://github.com/dalance/procs/compare/v0.10.2...v0.10.3) - 2020-05-11

* [Changed] Branch filtering of tree view [#59](https://github.com/dalance/procs/issues/59)

## [v0.10.2](https://github.com/dalance/procs/compare/v0.10.1...v0.10.2) - 2020-05-11

* [Changed] Enable XDG config path on macOS [#58](https://github.com/dalance/procs/issues/58)

## [v0.10.1](https://github.com/dalance/procs/compare/v0.10.0...v0.10.1) - 2020-05-01

* [Changed] Enable LTO [#56](https://github.com/dalance/procs/issues/56)

## [v0.10.0](https://github.com/dalance/procs/compare/v0.9.20...v0.10.0) - 2020-04-20

* [Added] header config [#54](https://github.com/dalance/procs/issues/54)
* [Changed] simplify default config [#55](https://github.com/dalance/procs/issues/55)

## [v0.9.20](https://github.com/dalance/procs/compare/v0.9.19...v0.9.20) - 2020-03-13

* [Added] Tree color config [#50](https://github.com/dalance/procs/issues/50)
* [Added] Black color and style [#49](https://github.com/dalance/procs/issues/49)
* [Fixed] broken pipe error

## [v0.9.19](https://github.com/dalance/procs/compare/v0.9.18...v0.9.19) - 2020-03-08

## [v0.9.18](https://github.com/dalance/procs/compare/v0.9.17...v0.9.18) - 2020-03-05

## [v0.9.17](https://github.com/dalance/procs/compare/v0.9.16...v0.9.17) - 2020-03-05

* [Changed] update proc-macro-error-attr [#45](https://github.com/dalance/procs/issues/45)

## [v0.9.16](https://github.com/dalance/procs/compare/v0.9.15...v0.9.16) - 2020-03-02

## [v0.9.15](https://github.com/dalance/procs/compare/v0.9.14...v0.9.15) - 2020-03-02

## [v0.9.14](https://github.com/dalance/procs/compare/v0.9.13...v0.9.14) - 2020-03-02

## [v0.9.13](https://github.com/dalance/procs/compare/v0.9.12...v0.9.13) - 2020-03-02

* [Fixed] garbage lines in watch mode

## [v0.9.12](https://github.com/dalance/procs/compare/v0.9.11...v0.9.12) - 2020-02-25

* [Fixed] separator's meaningless sort [#42](https://github.com/dalance/procs/issues/42)

## [v0.9.11](https://github.com/dalance/procs/compare/v0.9.10...v0.9.11) - 2020-02-16

## [v0.9.10](https://github.com/dalance/procs/compare/v0.9.9...v0.9.10) - 2020-02-16

* [Added] cargo feature to build without docker dependencies [#41](https://github.com/dalance/procs/issues/41)
* [Changed] remove unmaintained crates [#41](https://github.com/dalance/procs/issues/41)
* [Fixed] garbage characters in watch mode

## [v0.9.9](https://github.com/dalance/procs/compare/v0.9.8...v0.9.9) - 2020-02-12

## [v0.9.8](https://github.com/dalance/procs/compare/v0.9.7...v0.9.8) - 2020-02-12

## [v0.9.7](https://github.com/dalance/procs/compare/v0.9.6...v0.9.7) - 2020-02-12

* [Added] widths of columns are adjusted over iteration in watch mode
* [Fixed] suppress flicker in watch mode

## [v0.9.6](https://github.com/dalance/procs/compare/v0.9.5...v0.9.6) - 2020-02-05

* [Changed] --watch and --watch-interval option [#36](https://github.com/dalance/procs/issues/36)

## [v0.9.5](https://github.com/dalance/procs/compare/v0.9.4...v0.9.5) - 2020-01-30

* [Fixed] Remove --suid to fix security vulnerability (arbitrary command execution by root) [#38](https://github.com/dalance/procs/issues/38)

## [v0.9.4](https://github.com/dalance/procs/compare/v0.9.3...v0.9.4) - 2020-01-29

## [v0.9.3](https://github.com/dalance/procs/compare/v0.9.2...v0.9.3) - 2020-01-29

* [Fixed] tree view with filter [#37](https://github.com/dalance/procs/issues/37)

## [v0.9.2](https://github.com/dalance/procs/compare/v0.9.1...v0.9.2) - 2020-01-26

* [Changed] update console to v0.9.2 [#34](https://github.com/dalance/procs/issues/34)
* [Fixed] usage_mem overflow
* [Fixed] Ctrl-C is ignored on Windows [#35](https://github.com/dalance/procs/issues/35)

## [v0.9.1](https://github.com/dalance/procs/compare/v0.9.0...v0.9.1) - 2020-01-24

* [Fixed] clear screen at entering watch mode

## [v0.9.0](https://github.com/dalance/procs/compare/v0.8.16...v0.9.0) - 2020-01-23

* [Added] sort order changing by keyboard [#31](https://github.com/dalance/procs/issues/31)
* [Fixed] start_time slow down
* [Changed] from failure to anyhow

## [v0.8.16](https://github.com/dalance/procs/compare/v0.8.15...v0.8.16) - 2019-12-09

* [Fixed] refine PPID == PID case

## [v0.8.15](https://github.com/dalance/procs/compare/v0.8.14...v0.8.15) - 2019-12-09

* [Fixed] Tree view failure caused by PPID == PID

## [v0.8.14](https://github.com/dalance/procs/compare/v0.8.13...v0.8.14) - 2019-11-18

* [Changed] update procfs to v0.7.1

## [v0.8.13](https://github.com/dalance/procs/compare/v0.8.12...v0.8.13) - 2019-10-30

* [Changed] update procfs to v0.7.0

## [v0.8.12](https://github.com/dalance/procs/compare/v0.8.11...v0.8.12) - 2019-10-21

* [Added] UidLogin/UserLogin column
* [Changed] update procfs to v0.6.0

## [v0.8.11](https://github.com/dalance/procs/compare/v0.8.10...v0.8.11) - 2019-10-08

* [Changed] update procfs to v0.5.4

## [v0.8.10](https://github.com/dalance/procs/compare/v0.8.9...v0.8.10) - 2019-10-07

* [Changed] use libproc v0.5

## [v0.8.9](https://github.com/dalance/procs/compare/v0.8.8...v0.8.9) - 2019-09-05

* [Added] max_width/min_width option

## [v0.8.8](https://github.com/dalance/procs/compare/v0.8.7...v0.8.8) - 2019-06-25

* [Fixed] SIGSEGV at parallel test caused by non-threadsafe function call of rust-users

## [v0.8.7](https://github.com/dalance/procs/compare/v0.8.6...v0.8.7) - 2019-06-18

* [Fixed] watch mode panic on Windows
* [Changed] the crate to get executable from palaver to process_path
* [Changed] remove build.rs

## [v0.8.6](https://github.com/dalance/procs/compare/v0.8.5...v0.8.6) - 2019-06-10

* [Fixed] compile failure on i686

## [v0.8.5](https://github.com/dalance/procs/compare/v0.8.4...v0.8.5) - 2019-05-08

* [Fixed] usage_cpu calculation mistake when interval is larger than 1s.

## [v0.8.4](https://github.com/dalance/procs/compare/v0.8.3...v0.8.4) - 2019-05-07

* [Added] suid option
* [Fixed] some characters remain over refresh in watch mode

## [v0.8.3](https://github.com/dalance/procs/compare/v0.8.2...v0.8.3) - 2019-05-03

* [Fixed] panic caused by --tree and --sort

## [v0.8.2](https://github.com/dalance/procs/compare/v0.8.1...v0.8.2) - 2019-04-30

* [Fixed] panic caused by procfs

## [v0.8.1](https://github.com/dalance/procs/compare/v0.8.0...v0.8.1) - 2019-04-03

* [Fixed] watch mode with search is broken

## [v0.8.0](https://github.com/dalance/procs/compare/v0.7.6...v0.8.0) - 2019-04-03

* [Added] tree view

## [v0.7.6](https://github.com/dalance/procs/compare/v0.7.5...v0.7.6) - 2019-03-22

* [Fixed] show process list on --config and --list [#15](https://github.com/dalance/procs/pull/15)

## [v0.7.5](https://github.com/dalance/procs/compare/v0.7.4...v0.7.5) - 2019-03-21

* [Changed] use OS-specific location for the configuration file [#14](https://github.com/dalance/procs/pull/14)

## [v0.7.4](https://github.com/dalance/procs/compare/v0.6.0...v0.7.4) - 2019-03-16

* [Added] windows support
* [Changed] fast exit of watch mode

## [v0.6.0](https://github.com/dalance/procs/compare/v0.5.8...v0.6.0) - 2019-03-07

* [Added] watch mode
* [Fixed] panic by truncate inside multi-byte unicode character

## [v0.5.8](https://github.com/dalance/procs/compare/v0.5.7...v0.5.8) - 2019-03-06

* [Added] column description to `--list` output
* [Changed] sort indicator refine
* [Fixed] wrong column width calculation about full-width characters
* [Fixed] wrong `By*` style on center/right aligned column

## [v0.5.7](https://github.com/dalance/procs/compare/v0.5.6...v0.5.7) - 2019-03-05

* [Added] separator option to `~/.procs.toml` setting
* [Added] `--list` option to show column kind list
* [Added] Slot column to insert column by `--insert` oprion
* [Added] Sort indicator
* [Changed] the first decimal place of day/year in CpuTime is shown
* [Changed] default separator from "|" to "│" ( U+2502:Box Drawings Light Vertical )
* [Changed] eip/esp/sig* format to 16 hex digits
* [Changed] sort keyword is matched with column kind
* [Fixed] unmatched `--sort*` affects sort order

## [v0.5.6](https://github.com/dalance/procs/compare/v0.5.5...v0.5.6) - 2019-03-01

* [Added] Ssb column
* [Added] sort option

## [v0.5.5](https://github.com/dalance/procs/compare/v0.5.4...v0.5.5) - 2019-02-28

* [Added] logical operation for search keywords
* [Changed] default logical operation for search keywords from OR to AND

## [v0.5.4](https://github.com/dalance/procs/compare/v0.5.3...v0.5.4) - 2019-02-27

* [Added] text align option

## [v0.5.3](https://github.com/dalance/procs/compare/v0.5.2...v0.5.3) - 2019-02-27

* [Fixed] panic by overflow

## [v0.5.2](https://github.com/dalance/procs/compare/v0.5.1...v0.5.2) - 2019-02-25

* [Fixed] `cargo install` failure on macOS

## [v0.5.1](https://github.com/dalance/procs/compare/v0.5.0...v0.5.1) - 2019-02-24

* [Fixed] CI issue

## [v0.5.0](https://github.com/dalance/procs/compare/v0.4.8...v0.5.0) - 2019-02-23

* [Added] macOS support
* [Added] ContextSw/Gid*/Group*/Policy/Sig*/Uid*/User* column

## [v0.4.8](https://github.com/dalance/procs/compare/v0.4.7...v0.4.8) - 2019-02-21

* [Added] `color_mode` option to `~/.procs.toml` setting
* [Added] `--pager` commandline option
* [Fixed] pager command of `~/.procs.toml` is not affected

## [v0.4.7](https://github.com/dalance/procs/compare/v0.4.6...v0.4.7) - 2019-02-18

* [Fixed] panic caused by zombie process
* [Fixed] build failure on Rust 1.31.1

## [v0.4.6](https://github.com/dalance/procs/compare/v0.4.5...v0.4.6) - 2019-02-16

* [Fixed] default pager option is not affected

## [v0.4.5](https://github.com/dalance/procs/compare/v0.4.4...v0.4.5) - 2019-02-14

* [Added] Vm*/Wchan column
* [Changed] `VmPeak` is added to default

## [v0.4.4](https://github.com/dalance/procs/compare/v0.4.3...v0.4.4) - 2019-02-11

* [Added] `cut_to_*` options to `~/.procs.toml` setting
* [Changed] default pager is changed to `less -SR`

## [v0.4.3](https://github.com/dalance/procs/compare/v0.4.2...v0.4.3) - 2019-02-07

* [Added] Eip/Esp/MajFlt/MinFlt/Nice/Ppid/Priority/Processor/RtPriority column

## [v0.4.2](https://github.com/dalance/procs/compare/v0.4.1...v0.4.2) - 2019-02-06

* [Changed] default pager is changed to `less`

## [v0.4.1](https://github.com/dalance/procs/compare/v0.4.0...v0.4.1) - 2019-02-06

* [Fixed] failure of text width calculation with tab character

## [v0.4.0](https://github.com/dalance/procs/compare/v0.3.5...v0.4.0) - 2019-02-06

* [Added] pager support
* [Fixed] pipe broken error
