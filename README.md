# AmebaZ2 / AmebaZII

Open Source Implementation of OTA and Flash Generation for the AmebaZ2 SDK in Rust

> [!CAUTION]
> This repository is an unofficial implementation based on reverse engineering of the official SDK and previous work from [ltchiptool](https://github.com/libretiny-eu/ltchiptool).

*Documentation and usage information are a work in progress!*


## Usage

Currently, there are two main functionalities:

- `ota`: Work with OTA/update images.
- `flash`: Work with complete flash images.

### OTA

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

## Disclaimer

This repository and its associated tools are not affiliated with, endorsed by, or connected to AmebaIoT or any of its parent companies. The tools and research presented here are for educational and informational purposes only. Use them at your own risk. The author(s) of this repository do not take responsibility for any damage, data loss, or other issues caused by using the tools provided.


## License
Distributed under the MIT License. See LICENSE for more information.