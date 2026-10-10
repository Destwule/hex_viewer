# hex_viewer

A small command-line hex dump tool written in Rust, in the spirit of `xxd` and `hexdump`.
It prints a file (or standard input) as rows of hex bytes, with the byte offset on the left
and a printable-ASCII view on the right.

```
00000000: 48 65 6c 6c 6f 2c 20 77 6f 72 6c 64 21 0a        Hello, world!.
```

## Features

- Offset column, hex bytes and an ASCII sidebar
- Choose how many bytes go on each row
- Start at any byte offset, and stop after a given number of bytes
- Uppercase or lowercase hex
- Autoskip: collapse runs of all-zero rows into a single `*`
- Reads from a file or from standard input
- Optional output file instead of the terminal
- Streams the input in chunks, so large files are not loaded into memory
- Only uses portable `std::fs` / `std::io`, with no OS-specific code

## Build

```
cargo build --release
```

The binary ends up in `target/release/`. You can also run it directly:

```
cargo run --release -- <options>
```

## Usage

```
hex_viewer [OPTIONS] [INPUT] [OUTPUT]
```

- `INPUT`: file to dump. If you leave it out, the program reads from standard input.
- `OUTPUT`: file to write the dump to. If you leave it out, the dump goes to the terminal.
  An existing output file is overwritten.

### Options

| Option | Description | Default |
|---|---|---|
| `-c`, `--cols <N>` | Bytes per row (must be at least 1) | `16` |
| `-l`, `--len <N>` | Stop after `N` bytes | whole input |
| `-s`, `--seek <N>` | Skip the first `N` bytes. Offsets still show the real position in the file | `0` |
| `-u`, `--uppercase` | Print hex digits as `AB` instead of `ab` | off |
| `-a`, `--autoskip` | Collapse repeated all-zero rows into one `*` line | off |
| `--help` | Show the help text | |

## Examples

Dump a whole file:

```
hex_viewer disk.img
```

Eight bytes per row:

```
printf 'Hello, world!\n' | hex_viewer -c 8
```

```
00000000: 48 65 6c 6c 6f 2c 20 77  Hello, w
00000008: 6f 72 6c 64 21 0a        orld!.
```

Look at one region: start at byte 512 and show 64 bytes.

```
hex_viewer -s 512 -l 64 disk.img
```

Skip over long zero-filled gaps in a disk image:

```
hex_viewer -a disk.img
```

Write the dump to a file:

```
hex_viewer firmware.bin firmware.txt
```

Read from a pipe:

```
cat firmware.bin | hex_viewer
```

## Output format

Each row looks like this:

```
00000010: 03 00 3e 00 01 00 00 00 40 10 00 00 00 00 00 00  ..>.....@.......
```

- The first column is the byte offset of the row, in hex.
- Next come the bytes of the row, two hex digits each.
- The last column shows each byte as a character. Bytes from 32 to 126 (printable ASCII)
  show as themselves, and every other byte shows as `.`.
- The final row may be shorter than the others. It is padded so the ASCII column stays aligned.

With `--autoskip`, the first all-zero row in a run is printed normally. The rest of the run is
replaced by a single `*`, and the next row with data shows its real offset.

## Notes and limitations

- `--seek` and `--len` currently apply to files only. They are ignored when reading from standard input.

## Redox OS

The tool uses only portable standard-library calls, so it builds for [Redox OS](https://www.redox-os.org/)
with [`redoxer`](https://gitlab.redox-os.org/redox-os/redoxer):

```
redoxer build
redoxer run
```

Early versions of this tool were run this way inside a Redox VM.