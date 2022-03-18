# tpd
Typo detector for english words. The default dictionary words from [here](https://github.com/first20hours/google-10000-english/blob/master/20k.txt)

If you want to add more words you can add config file at `$HOME/.config/tpd/dictionary`.

## Usage

```bash
tpd 0.1.0
Typo detector for english words

USAGE:
    tpd [OPTIONS] [FILE_PATH] [SUBCOMMAND]

ARGS:
    <FILE_PATH>    File path to read

OPTIONS:
    -h, --help       Print help information
    -s, --suggest    Suggest fix for typo
    -V, --version    Print version information

SUBCOMMANDS:
    add     Add custom word to dictionary
    help    Print this message or the help of the given subcommand(s)
```

## Demo

![demo](demo.gif)
