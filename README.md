# srtshift
Shift and trim SubRip files

## Compiling

Install rust and cargo and run:

```
cargo build
```

## Running

```
USAGE:
    srtshift [OPTIONS] --input <input> --output <output> --shift <shift>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --cutafter <cutafter>    Cut timestamps after [+/-]hh:mm:ss,xxx
    -i, --input <input>          Input file
    -o, --output <output>        Output file
    -s, --shift <shift>          Shift timestamps [+/-]hh:mm:ss,xxx
```
