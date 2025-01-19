# Flash Command Line Interface (no device interaction)

## TL;DR

```bash
# parse flash and display info
amebazii flash parse [FILE]

# extract all partitions from flash image
amebazii flash split [FILE] [OUTDIR]

# build flash image from existing partitions
amebazii flash combine -p [PARTTAB] [OUTFILE]
```

## Parsing

The `flash` subcommand can parse and extract all partitions from a raw flash image (which can be extracted using [ltchiptool](https://github.com/libretiny-eu/ltchiptool)).

To parse the flash image (use `--pt-only` to parse only the partition table)
```
$ amebazii flash parse --pt-only assets/partition.bin
===================================== Partition Table =====================================
Public Keys:
  [0] - "68513ef83e396b12ba059a900f36b6d31d11fe1c5d25eb8aa7c550307f9c2405"
        Note: this partition table uses the default encryption key
  [1] - "48ad23ddbdac9e65719db7d394d44d62820d19e50d68376774237e98d2305e6a"
        Note: this partition table uses the default hash key

Signature: (using default hash key)
  - "b0bd1cb0217cc372dca7d999c4f6451c10405cee97558cdb5d9ae8d3ebd76fa2" OK

User Data:
  - UserExt: "ffffffffffffffffffffffff"
  - UserBin: valid, length=256
  - Fw1 Index: 1
  - Fw2 Index: 2

Records:
  [0] - Type: Boot (offset: 0x004000, length: 0x008000)
      - HashKey: <not set>

  [1] - Type: Fw1 (offset: 0x00c000, length: 0x0f8000)
      - HashKey: "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e5f"
        Note: this partition uses a default hash key

  [2] - Type: Fw2 (offset: 0x104000, length: 0x0f8000)
      - HashKey: "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e5f"
        Note: this partition uses a default hash key

===========================================================================================
```

## Extracting Partitions

To extract all partitions:

```bash
$ amebazii flash split --include-common flash_is.bin flash_out
[0] Record (Boot)
  - Offset: 0x004000, Length: 0x008000)
[1] Record (Fw1)
  - Offset: 0x00c000, Length: 0x0f8000)
[2] Record (Fw2)
  - Start address 0x104000 is larger than file size 0x082544 - skipping...
```

The target directory will store `partition.bin` for the partition table and all other
partitions mentioned within the partition table.

```
$ ls flash_out
boot.bin  fw1.bin  partition.bin  sysdata.bin
```

## Building a complete Flash Image (experimental)

Example:
```bash
$ amebazii flash combine -p partition.bin --pt-has-calibpat \
    --fw1 fw1.bin \
    --fw2 fw2.bin \
    --boot bootloader.bin \
    --user userdata.bin \
    new_flash.bin`
[Fw1] => Status: Ok
[Fw2] => Status: Ok
[User] => Status: Ok
[Boot] => Status: Ok
```