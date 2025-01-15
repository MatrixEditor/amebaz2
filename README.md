# AmebaZ2 / AmebaZII

Open Source Implementation of OTA and Flash Generation for the AmebaZ2 SDK in Rust

> [!CAUTION]
> This repository is an unofficial implementation based on reverse engineering of the official SDK and previous work from [ltchiptool](https://github.com/libretiny-eu/ltchiptool).

*Documentation and usage information are a work in progress!*

Features:

* Parse OTA and Flash images (w/ extraction support)
* [_**Relink**_](#relinking) existing OTA images back to their compiled application binary (ELF) 🎊
* Build a partition table and system data partition

## Usage

Currently, there are two main functionalities:

- `ota`: Work with OTA/update images.
- `flash`: Work with complete flash images.

### OTA

#### Parsing

The current implementation allows parsing and verification of images:

```bash
$ amebazii ota parse [FILE]
```

Using the example binary from the [/assets](/assets/) directory, the output should be similar to the following:
```text
============================================ OTA Image =============================================
Public Keys:
  [Hash Public Key] - "68513ef83e396b12ba059a900f36b6d31d11fe1c5d25eb8aa7c550307f9c2405"
  [0] -  <not set>
  [1] -  <not set>
  [2] -  <not set>
  [3] -  <not set>
  [4] -  <not set>

OTA-Signature: (using default hash key)
  - "2b763781b4199d29089a933b3aecd8d65124b1f2cbf8fa3aa17c84ff37be355e" OK
  - Checksum: 0x327dfa5 OK
====================================================================================================

--------------------------------------------- Subimage ---------------------------------------------
Header:
  - Type: FHWSS
  - Size: 0x00002ae0
  - Serial: 100

User Keys:
  [0] - <not set>
  [1] - "bb0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"

Security:
  - Encryption: disabled
  - Hashing: enabled (Sha256)
    - "79ba081144b693fcb04155539bee4f984ee0ca7d490c69ed6375801db1e81c2d" OK

Sections:
  [0] - SRAM (length: 0x00002a20, load: 0x10000480, entry: 0x10000480)
----------------------------------------------------------------------------------------------------
[...]
```

> [!NOTE]
> Use `--boot` to parse a boot image extracted from a flash image.

#### Relinking

*Currently searching for the appropriate wording, but relinking seems to describe the process very well.*

Based on the information from the OTA we can *relink* all sections back to a valid ELF binary using the
power of [gimli-rs/object](https://github.com/gimli-rs/object).

```bash
amebazii ota relink -s ./section_data --cap-length ./assets/fw1.bin ./fw1.elf
```

The output will be something similar to this:
```text
RAM/FHWS:
  [0] Secion: SRAM
      - RAM function table... OK
      - RAM image signature... OK
      - DTCM RAM... OK
XIP:
  [0] Secion: XIP_C
      - XIP code cipher section... OK
  [1] Secion: XIP_P
      - XIP code plaintext section (rodata)... OK
      - RAM vector table... OK

Program Headers:
  Type  Offset   VirtAddr   PhysAddr   FileSiz MemSiz  Flg Align   Info
  LOAD  0x000000 0x00000000 0x00000000 0x71b38 0x71b38 R   0x10000 Standard sections
  LOAD  0x010000 0x10000000 0x10000000 0x000a0 0x000a0 RW  0x10000 RAM vector table
  LOAD  0x010480 0x10000480 0x10000480 0x00070 0x00070 RW  0x10000 RAM function table
  LOAD  0x0104f0 0x100004f0 0x100004f0 0x00010 0x00010 R   0x10000 RAM image signature
  LOAD  0x010500 0x10000500 0x10000500 0x02980 0x02980 RWE 0x10000 RAM text and rodata
  LOAD  0x04fa00 0x9b000140 0x9b000140 0x53768 0x53768 RWE 0x10000 XIP.C rodata, text
  LOAD  0x84f8c0 0x9b800140 0x9b800140 0x1e3d0 0x1e3d0 RW  0x10000 XIP.P rodata, text
```

> [!NOTE]
> Even though, one could put that file into Ghidra and start inspecting the code,
> it won't load all segments correctly --> WIP. Currently, the standard segments
> with custom labels doesn't get imported properly. All other segments work as
> expected.

#### Extraction

You can also dump each subimage manually by using `dump`:

```bash
amebazii ota dump ./assets/fw1.bin -I [INDEX] [--section INDEX] [OUTDIR/FILE]
```

Only specifying the subimage index will dump all sections within that subimage to the
given destination directory. Using the extra option `--section` you can dump a specific
section to an output file.

For example:
```bash
$ amebazii ota dump ./assets/fw1.bin -I 0 --section 0 ./test.bin
[0] Subimage: FHWSS
    [0] Section: SRAM (Length: 0x00002a20, LoadAddress: 0x10000480, EntryAddress: 0x10000480)
```

### Flash

The `flash` subcommand can parse and extract all partitions from a raw flash image (which can be extracted using [ltchiptool](https://github.com/libretiny-eu/ltchiptool)).

* To parse the flash image (use `--pt-only` to parse only the partition table):
    ```bash
    $ amebaz2 flash parse --pt-only [FILE]
    ```
    The output will show the contents of the partition table:
    ```text
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

* To extract all partitions:
    ```bash
    $ amebazii flash split [FILE] [OUTDIR]
    ```
    The target directory will store `partition.bin` for the partition table and all other
    partitions mentioned within the partition table.

#### Combine Partitions

*TODO*


## Disclaimer

This repository and its associated tools are not affiliated with, endorsed by, or connected to AmebaIoT or any of its parent companies. The tools and research presented here are for educational and informational purposes only. Use them at your own risk. The author(s) of this repository do not take responsibility for any damage, data loss, or other issues caused by using the tools provided.


## License
Distributed under the MIT License. See LICENSE for more information.