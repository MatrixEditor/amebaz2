# OTA Firmware Command Line Interface

## TL;DR

```bash
# parse image and get info (validates signatures)
amebazii ota parse [OTAFILE]

# extract sections from subimage to directory
amebazii ota dump -I 0 [OTAFILE] [DIR]

# recreate application binary for reversing
amebazii ota relink -c [OTAFILE] [OUTFILE]

# sign existing image using custom key
amebazii ota resign [OTAFILE] -k [KEY] [OUTFILE]
```


## Parsing OTA Images (and Boot Images)

The current implementation allows parsing and verification of OTA firmware update images and boot images.

**SYNOPSIS**
```bash
amebazii ota parse [--boot] <FILE>
```

Using the example binary from the [/assets](/assets/) directory, the output should be similar to the following:

```
$ amebazii ota parse assets/fw1.bin
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

The `--boot` flag can be used to parse bootloader images taken from a complete flash image.
However, verification will be done using the default hash key:

```
$ amebazii ota parse --boot bootloader.bin
============================================ Bootloader ============================================
Header:
  - Type: Boot
  - Size: 0x00004020
  - Serial: 0

Security:
  - Encryption: disabled
  - Hash: 4fe772b044e066fed83d5312edca75ea8ea87ec5e46f32a9a9d95990b3192fd6 OK

Sections:
  [0] - Bootloader (length: 0x00004000, load: 0x10038100, entry: 0x10038100)
====================================================================================================
```

## Relinking

*Currently searching for the appropriate wording, but relinking seems to describe the process very well.*

Based on the information from the OTA we can *relink* all sections back to a valid ELF binary using the
power of [gimli-rs/object](https://github.com/gimli-rs/object).

```
$ amebazii ota relink --cap-length ./assets/fw1.bin ./fw1.elf
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
  LOAD  0x000000 0x00000000 0x00000000 0x00000 0x00000 R   0x10000 Standard sections
  LOAD  0x010000 0x10000000 0x10000000 0x000a0 0x000a0 RW  0x10000 RAM vector table
  LOAD  0x010480 0x10000480 0x10000480 0x00070 0x00070 RW  0x10000 RAM function table
  LOAD  0x0104f0 0x100004f0 0x100004f0 0x00010 0x00010 R   0x10000 RAM image signature
  LOAD  0x010500 0x10000500 0x10000500 0x02980 0x02980 RWE 0x10000 RAM text and rodata
  LOAD  0x04fa00 0x9b000140 0x9b000140 0x53768 0x53768 RWE 0x10000 XIP.C rodata, text
  LOAD  0x84f8c0 0x9b800140 0x9b800140 0x1e3d0 0x1e3d0 RW  0x10000 XIP.P rodata, text
```

<div class="warning">

Even though, one could put that file into Ghidra and start inspecting the code,
it won't load all segments correctly --> WIP. Currently, the standard segments
with custom labels doesn't get imported properly. All other segments work as
expected.

</div>

To save all sections that will be copied into the final binary, use `-s/--save-intermediate <DIR>`.


## Extraction

You can also dump each subimage manually by using `dump`:

```
$ amebazii ota dump ./assets/fw1.bin -I [INDEX] [--section INDEX] [OUTDIR/FILE]
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

## Resigning Firmware Files

To apply a custom key for signature generation, firmware images can be *resigned*. If you omit the key
argument, the default signing key will be applied.

```
$ amebazii ota resign [OTAFILE] -k [KEY] [OUTFILE]
[0] Subimage: FHWSS
 - Old signature: 77223f302b796db80630994fc5e26e280653d6e8d2c98843382686cc3c917b01
 - New signature: e2127dcf597cb0202e8f6b31406481b43d9b566312af71433de69adeb5036ee2

[1] Subimage: Xip
 - Old signature: cbdaeebeee049f4189f058db5c93cb6bac35333c4cdf12f5c8b3696e4452b23a
 - New signature: ed734027fedc74c468f3ee630a1a3aa7a497f4f94663430f0e92c2e1d7ab9b7d

[2] Subimage: Xip
 - Old signature: 8700a6c01dd5850202186a35a86824cf83f6825c6ff86edf989b156de1b12d74
 - New signature: c5a9e71c36696ee332b5c431e9a9f28845094c17885e0ad125980d66986a3744

[OTA] Old signature: 11e82d6903026c5f891cbaebc98f6f334979a07605e0d1002ec7f071109ca7d0
[OTA] New signature: 43575e10e43e6122892717f614d3d0badc37ba81067d5579deadadaa2f107702
```