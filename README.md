# tl-zork

A tiny Zork-style text adventure, played live on stage as a 5-minute lightning talk at
TechLancaster on September 24, 2026.

## Install

```sh
cargo install tl-zork
```

Or from source:

```sh
git clone https://github.com/zachfedor/tl-zork
cd tl-zork
cargo run
```

## Play

Type commands at the `>` prompt, like `look`, `north`, `take coffee`, `inventory`, or
`time`. The clock starts the moment the game launches, and it doesn't pause.

## Development

```sh
cargo test     # tests simulate real time
cargo clippy --all-targets
cargo fmt
```

## Give a talk

TechLancaster, Thursday, October 22, 2026, at West Art, 816 Buchanan Ave, Lancaster, PA.

Sign up: https://bit.ly/tl-lightning-signup-2026

