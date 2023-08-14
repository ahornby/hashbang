# hashbang - download, cache and launch commands

Hashbang downloads, caches and launches other commands. Typical uses for this tool might be distributing developer tool binary or script updates (e.g. an internal setup script, or a picking  up latest bazel, sapling, or buck2 releases), but it can download, cache and run anything.

Hashbang can currently wrap locally installed tools, download from plain old urls, and soon can track github releases, with logic to allow you to select the source based on the target platform.

The name comes from `#!` at the start of unix scripts where one can specify the interpreter to use for the script.  Hashbang uses this to make its configs directly runnable in place of the tools its launching.

## Installation

To build and install from [crates.io](https://crates.io/crates/hashbang), run:
```shell
$ cargo install hashbang
```

If you don't have cargo installed,  install via [rustup.rs](https://rustup.rs/)

## Running hashbang

In general you run a hashbang config like you would the tool it is launching,  much like you would a traditional wrapper script or functon.  Behind the scenes hashbang caches, downloads, and extracts archives; from which it runs binaries.

You once you have installed hashbang as above you can try out one of the example configs to download, cache and run bazel7 like so:
```shell
$ ./examples/bazel7.toml
```

If you want to make that the default way you find and discovery bazel you could copy to a directory on your PATH, e.g. `cp ./examples/bazel7.toml ~/bin/bazel`

Or try out buck2 with:
```shell
$ ./examples/buck2-url.toml
```

Similarly you could `cp ./examples/buck2-url.toml ~/bin/buck2` if you wanted hashbang to manage your buck2 upgrades for you.

If you only want interative use, could also use an alias, e.g. `alias buck2="$HOME/local/hashbang/examples/buck2-url.toml"`

## Environment variables

`HASHBANG_CONFIG_URL` a file:, data: or https: url to load config from. e.g. to run a config explicitly:
```shell
 $ HASHBANG_CONFIG_URL=file:./examples/buck2-url.toml hashbang --version
```

`HASHBANG_BINARY` Optional: tells hashbang which binary to run if there are multiple in the config.

## Config syntax

`hashbang` config is in toml, and allows you to specify which archives to download and cache and which binaries to run from those cached archives.

At the top level the only current entry is `name`, allowing you to name your config.

For example config and how to use hashbang as a #! interpreter see [examples](./tests/data/),  which has examples using a plain old URL, and those using github releases.

### Platform selection

The `[[platform]]` sections of the hashbang config are checked in order and specific a `target_spec`, thus allowing you to first have a specific case like `aarch64-apple-darwin`, and then a fallback like `cfg(unix)` if the more specific cases don't match. Its the sample cfg syntax as used by cargo,  you can find more details of supported syntax in the [target-spec crate](https://crates.io/crates/target-spec) that supplies the functionality.

## Related work

[bazelisk](https://github.com/bazelbuild/bazelisk/blob/master/README.md) is a bazel download, cache and run tool.  Its mature and has many users. One of the reasons I started hashbang is that bazelisk distribution is usually npm based so one has to install go or npm as a prerequisite (brew uses npm), and I wanted a lighter laucher.

[buckle](https://crates.io/crates/buckle) is a buck2 download, cache and run tool.  It specializes in buck2 only.  One of the reasons I started hashbang is that buckle only handles buck2, and a lot of the concerns of download/cache/run seem like they apply across many things we might want to run.

[dotslash](https://crates.io/search?q=dotslash) is a cross platform binary fetcher and launcher used internally in Meta that, like hashbang, is binary agnostic and usable to launch all kinds of tools. The public crate is a placeholder incase dotslash is open sourced in future. 

## FAQs
Frequently asked of myself in any case.

### Why not call this tool shebang?
Historically I think I heard shebang more than hashbang but neither were super common terms really. Deciding factor was that the shebang crate name was taken!

### Why [toml](https://toml.io/) config rather than yaml, json etc?
Mostly because cargo and other rust based tooling uses toml. Secondary reasons: YAML historically had a bad rap due to the implicit conversions causing the [norway problem](https://hitchdev.com/strictyaml/why/implicit-typing-removed/) (norway is fixed in YAML 1.2 based tools); and JSON parsers often don't allow comments.

### Why not wait for dotslash to be open sourced?
Hard to know if or when that will happen,  so as well as praying for help, also rowing for shore!  Many other launchers are specific to a single tool at the moment, and I want to demonstrate that doesn't need to be the case and hopefully produce something useful in the process. 

### Why do you call things "binaries" when they can be shell scripts or other executable formats
It comes from a preference for shipping self contained statically linked binaries. Inside tech companies "binary" evolved from that as a useful shorthand for "self contained executable" even if that executable is a shell script, [python .par/.xar bundle](https://engineering.fb.com/2018/07/13/data-infrastructure/xars-a-more-efficient-open-source-system-for-self-contained-executables/) etc.

### Should I use hashbang instead of a package manager like brew, dnf, or apt?
If you can package as a self contained binary, maybe.  If you need to install dependencies e.g. you dynamically link to a C library and can't change to static, or you don't package all your python deps into a par/xar, you are likely better off with brew, dnf, or apt.

## Contributing

Please raise PRs against [the repo](https://github.com/ahornby/hashbang) including tests. You might find https://sapling-scm.com/ and https://reviewstack.dev/ help keep to "one PR == one commit" to produce small easily reviewable changes.

## License

Hashbang is copyright Alex Hornby <alex@hornby.org.uk> and released under the terms of the [MIT LICENSE](./LICENSE).
