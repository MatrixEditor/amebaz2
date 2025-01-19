# AmebaZ2 / AmebaZII

Open Source Implementation of OTA and Flash Generation for the AmebaZ2 SDK in Rust

> [!CAUTION]
> This repository is an unofficial implementation based on reverse engineering of the official SDK and previous work from [ltchiptool](https://github.com/libretiny-eu/ltchiptool).

*Documentation and usage information are a work in progress!*

Features:

* Parse OTA and Flash images (w/ extraction support)
* [_**Relink**_](src/amebazii/doc/cmd_ota.md#relinking) existing OTA images back to their compiled application binary (ELF) 🎊
* [_**Resign**_](src/amebazii/doc/cmd_ota.md#resigning-firmware-files) existing OTA images using custom keys. 🎉
* Build a partition table and system data partition


Documentation for CLI has moved to [amebazii/doc](src/amebazii/doc/) and general API docs are avaiable on Github-Pages [here >>](https://matrixeditor.github.io/amebaz2/amebazii/).

## Disclaimer

This repository and its associated tools are not affiliated with, endorsed by, or connected to AmebaIoT or any of its parent companies. The tools and research presented here are for educational and informational purposes only. Use them at your own risk. The author(s) of this repository do not take responsibility for any damage, data loss, or other issues caused by using the tools provided.


## License
Distributed under the MIT License. See LICENSE for more information.