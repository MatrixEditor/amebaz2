searchState.loadedDescShard("amebazii", 0, "See the CLI documentation.\nDocumentation\nChecks if the given data is valid by ensuring none of the …\nDefault keys used to sign and verify images/ partition …\nRead (skip) padding bytes in a reader.\nReads valid data from the reader into the target, ensuring …\nUtility functions for creating signatures and hashes.\nWrites padding to a binary stream to ensure that the next …\nWrites data to a stream.\n<code>write_padding!</code> - A macro to write padding bytes to a …\nA struct representing an array of bytes of a fixed size, …\nThe byte array that holds the data.\nCustom deserialization logic for <code>DataArray</code>. It reads the …\nReturns the argument unchanged.\nCreates a <code>DataArray</code> from a hexadecimal string.\nCalls <code>U::from(self)</code>.\nLoads the data from a file at the given path.\nCreates a new <code>DataArray</code> from a string that is either a …\nThe optional file path from which the data was loaded (if …\nCustom serialization logic for <code>DataArray</code>. If the data was …\nRepresents a single item in the partition table …\nRepresents the partition table configuration.\nReturns the default configuration for a <code>PartitionItemCfg</code> …\nReturns the default configuration for a <code>PartitionTableCfg</code> …\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nConverts a <code>PartitionItemCfg</code> instance into a <code>Record</code> …\nConverts a <code>PartitionTableCfg</code> instance into a <code>PartTab</code> …\nConfiguration for the system data, containing options for …\nBluetooth parameter data, stored as raw data.\nReturns the default configuration for <code>SystemDataCfg</code>.\nInformation related to the flash memory (e.g., flash ID, …\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nConfiguration for the old image trap, typically used to …\nThe address of the second OTA partition, if available.\nThe size of the second OTA partition, if available.\nConfiguration of the SPI interface, including IO mode and …\nSPI calibration configuration, stored as raw data.\nTries to convert <code>SystemDataCfg</code> into a <code>SystemData</code> instance.\nBaud rate for the UART logging interface.\nFlash Command Line Interface (no device interaction)\nOTA Firmware Command Line Interface\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nDefault second key used in the application subimage.\nDefault first key used in the boot image.\nRepresents a pair of cryptographic keys: a private key and …\nThe default Initialization Vector (IV) used for …\nDefault flash calibration pattern used in flash\nThe default hash key used to generate signatures for the …\nDefault cryptographic key pair for general use in …\nDefault hash key pair used for hashing operations (not …\nUnused key pair; not seen\nDefault key pair used for firmware signature generation …\nDefault IV used for encrypting/decrypting XIP section data.\nDefault AES encryption key used for encrypting/decrypting …\nReturns the argument unchanged.\nReturns a reference to the private key.\nReturns a reference to the public key.\nCalls <code>U::from(self)</code>.\nCreates a new <code>KeyPair</code> instance with the provided private …\nRepresents a memory address range, defined by a starting …\nInternal RAM for program data and text (Data and Text are …\nExtension RAM for heap, used as dynamic memory.\nExternal PSRAM for storing text, read-only data (RODATA), …\nRAM functions entry table, storing function entries in RAM.\nRAM image signature, used for storing the image signature …\nThe vector table, it must start with 256 bytes aligned …\nXIP Chiper section, where TEXT and RODATA sections <strong>can</strong> be …\nXIP Plaintext section, where RODATA is not encrypted.\nChecks if the provided address falls within the range …\nReturns the end address of the range.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nReturns the length of the address range, calculated as the …\nCreates a new <code>AddressRange</code> instance with a given start and …\nReturns the start address of the range.\nA trait for types that can provide their binary size.\n<code>DataRefType</code> is a type alias for an optional reference to a …\n<code>DataType</code> is a type alias for an optional fixed-size array …\nA trait for types that can be deserialized from a stream.\nNo value.\nNo value.\nSome value of type <code>T</code>.\nSome value of type <code>T</code>.\nA trait for types that can be serialized to a stream.\nReturns the binary size of the type in bytes.\nReads a type from a stream.\nConverts a hexadecimal string into a <code>DataType</code> array.\nConverts a <code>DataType</code> array into a hexadecimal string.\nReads data from a stream and populates the fields of the …\nSerializes an object into a vector of bytes.\nSerializes an object into a vector of bytes with an …\nTransfers a type’s data to a stream.\nWrites the type’s data to a stream.\nBoot partition (1).\nExport both keys (2).\nCalibration partition (5).\nSupported encryption algorithms.\nRepresents different flash sizes based on the …\nFirmware partition 1 (2).\nFirmware partition 2 (3).\nSupported various hash algorithms.\nEnum representing different image types.\nEnum representing different key export operations.\nExport the latest key (1).\nMain partition (8).\nNo key export operation (0).\nPartition table (0).\nEnum representing all different types of partitions. (as …\nReserved partition (9).\nEnum representing different section types in memory.\nRepresents different SPI I/O modes used for communication …\nRepresents different SPI clock speeds for communication …\nSystem partition (4).\nUser data partition (6).\nVariable partition (7).\nExecute-In-Place (XIP) contains the raw binary with all …\nAvailable sizes for XIP (Execute-In-Place) page remapping.\nComputes the hash of the provided buffer using the …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nConverts a 16-bit value into a <code>FlashSize</code> enum.\nReturns the argument unchanged.\nReturns the argument unchanged.\nConverts a 16-bit value into a <code>SpiIOMode</code> enum.\nConverts a 16-bit value into a <code>SpiSpeed</code> enum.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nReturns the size of the page in bytes for the given …\nAttempts to convert a <code>u8</code> value to an <code>ImageType</code> variant.\nTries to convert a <code>u8</code> value into a corresponding …\nAttempts to convert a <code>u8</code> value to an <code>XipPageRemapSize</code> …\nTries to convert a <code>u16</code> value to an <code>EncryptionAlgo</code> variant.\nTries to convert a <code>u16</code> value to a corresponding <code>HashAlgo</code> …\nAttempts to convert a <code>u8</code> value to the corresponding …\nTries to convert a <code>u8</code> value to the corresponding …\nRepresents a flash image, including calibration data and …\nRepresents different types of partitions in a flash image.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCreates a <code>Partition</code> from a <code>Record</code> and a reader stream.\nReturns a reference to the calibration pattern.\nReturns a mutable reference to the calibration pattern.\nRetrieves a partition by its type.\nChecks whether a partition of the specified type exists.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nReads the flash image from the provided reader.\nSets the boot partition with the given boot image.\nSets the first firmware partition with the provided …\nSets the second firmware partition with the given firmware …\nSets the partition for the specified type.\nSets the partition table with the provided partition table …\nSets the system partition with the provided system data.\nWrites the flash image to the provided writer.\nFirmware Security Table (FST)\nReturns the binary size of the <code>FST</code> structure in bytes.\nencryption algorithm (not supported)\nReturns the argument unchanged.\nReturns a reference to the cipher IV if it is set.\nReturns a reference to the cipher key if it is set.\nReturns a reference to the validation pattern used for the …\nThe hash algorithm used for hashing. Default is <code>Sha256</code>.\nCalls <code>U::from(self)</code>.\nChecks if the cipher key and IV are valid.\nReads the <code>FST</code> structure from a stream and parses its data.\nWrites the <code>FST</code> structure to a stream\n<code>EntryHeader</code> represents the header of a specific entry …\nGeneric image header.\nA struct representing the key block with two public keys:\nRepresents the header of a section in a binary image.\nReturns the binary size of the <code>KeyBlock</code> in bytes.\nReturns the binary size of the <code>ImageHeader</code> in bytes.\nReturns the binary size (in bytes) of the <code>SectionHeader</code> …\nReturns the binary size of the <code>EntryHeader</code> struct in bytes.\nCreates a default <code>KeyBlock</code> with both public keys …\nCreates a default <code>ImageHeader</code> instance with the following …\nReturns the default <code>SectionHeader</code> instance with predefined …\nReturns the default values for the <code>EntryHeader</code> struct.\nThe entry address, the address to which the system will …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nRetrieves the encryption public key.\nRetrieves a mutable reference to the encryption public key.\nRetrieves the hash public key.\nRetrieves a mutable reference to the hash public key.\nGets the first user key (<code>user_key1</code>).\nGets the second user key (<code>user_key2</code>).\nRetrieves the valid pattern.\nRetrieves the XIP IV.\nRetrieves the XIP key.\nChecks if there is a next image header.\nChecks if the section has a next section.\nThe type of the image.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nChecks if the encryption public key is valid.\nFlag indicating whether the image is encrypted.\nChecks if the hash public key is valid.\nChecks if the first user key (<code>user_key1</code>) is valid.\nChecks if the second user key (<code>user_key2</code>) is valid.\nThe length of the section in bytes.\nThe length of the entry in bytes.\nThe load address in memory where the entry will be loaded.\nOffset to the next image header.\nOffset to the next section.\nDeserializes a <code>KeyBlock</code> from a stream.\nReads an <code>ImageHeader</code> from a stream, populating its fields.\nReads the <code>SectionHeader</code> struct from a stream.\nReads an <code>EntryHeader</code> from the provided reader (e.g., a …\nIndicates whether Secure Copy Engine (SCE) is enabled for …\nThe type of the current section.\nThe size of the image segment in bytes.\nThe serial number associated with the image. (version …\nSets the first user key (<code>user_key1</code>) in the image header.\nSets the second user key (<code>user_key2</code>) in the image header.\nUser key 1, used for encryption\nUser key 2, used for encryption\nSerializes a <code>KeyBlock</code> to a stream.\nSerializes the <code>ImageHeader</code> struct to a stream.\nSerializes the <code>SectionHeader</code> struct to a stream.\nSerializes the <code>EntryHeader</code> struct to a stream.\nBlock size for XIP remapping.\nChecks if both the <code>xip_key</code> and <code>xip_iv</code> fields are valid.\nXIP (Execute-In-Place) page size and remapping setting.\nA trait that provides common functionality for image-like …\nContains encrypted data as a vector of bytes.\nA generic enum representing either encrypted or plain data.\nContains plain, unencrypted data of type <code>T</code>.\nReturns a mutable reference to the encrypted data if …\nReturns a mutable reference to the plain data if available.\nReturns a reference to the encrypted data if available.\nReturns a reference to the plain data if available.\nBuilds the signature for a given image.\nBuilds the segment size for a given image.\nComputes the segment size for the image.\nComputes the signature for the image using the provided …\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nReturns <code>true</code> if the data is encrypted.\nReturns <code>true</code> if the data is plain.\nReads the data from a stream, either encrypted or plain.\nSets the segment size for a given image.\nSets the signature for a given image.\nSets the segment size for the image.\nSets the signature for the image.\nWrites the data to a stream, either encrypted or plain.\nRepresents a boot image, including encryption public keys, …\nComputes the segment size for the BootImage.\nComputes the signature for the BootImage.\nCreates a new <code>BootImage</code> with default values.\nThe entry header, typically pointing to the start of the …\nReturns the argument unchanged.\nRetrieves the hash value associated with the boot image.\nRetrieves the text (code) content of the boot image.\nThe header of the boot image, containing general …\nCalls <code>U::from(self)</code>.\nReads a <code>BootImage</code> from a binary stream.\nSets the segment size for the BootImage.\nSets the signature for the BootImage.\nSets the text content of the boot image.\nWrites a <code>BootImage</code> to a binary stream.\nRepresents an OTA (Over-The-Air) image.\nRepresents a sub-image, including a header, FST (Firmware …\nAdds a new section to the sub-image.\nAdds a new subimage to the OTA image.\nComputes the checksum for the OTA image by writing it to a …\nBuilds the OTA image signature, which is the hash result …\nBuild the segment size for the SubImage.\nBuild the signature for the SubImage.\nA checksum value for verifying the integrity of the OTA …\nCalculates a checksum from a byte buffer by summing all …\nCalculates a checksum from a stream by reading the content …\nCreates a new <code>SubImage</code> with default values.\nCreates a default <code>OTAImage</code> with an empty keyblock, no …\nReturns the argument unchanged.\nReturns the argument unchanged.\nThe Firmware Security Table (FST) associated with the …\nReturns a reference to the hash of the sub-image.\nReturns the encryption public key from the keyblock, which …\nRetrieves a specific public key used for signature …\nReturns a reference to the section at the specified index, …\nReturns a mutable reference to the section at the …\nReturns a reference to the sections in the sub-image.\nReturns a mutable reference to the sections in the …\nRetrieves a specific subimage by its index.\nRetrieves a mutable reference to a specific subimage by …\nReturns a slice of the subimages contained in the OTA …\nReturns a mutable slice of the subimages contained in the …\nThe header of the sub-image containing general information …\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nThe key block containing cryptographic keys for encryption …\nReads the OTA signature from a stream and computes its …\nReads a <code>SubImage</code> from a binary stream.\nReads an <code>OTAImage</code> from a binary stream.\nRemoves the section at the specified index from the …\nRemoves a subimage from the OTA image at the specified …\nSets the OTA image signature, specifically the public …\nSet the segment size for the SubImage.\nSet the signature for the SubImage.\nReads the signature for this <code>SubImage</code> from a binary stream …\nUpdates the checksum field of the OTA image.\nUpdates the OTA image signature using the provided public …\nWrites a <code>SubImage</code> to a binary stream.\nWrites an <code>OTAImage</code> to a binary stream.\n…\n…\nRepresents a firmware partition record.\nRepresents the configuration of a hardware trap.\nAdds a new partition record to the partition table.\nReturns the size of the <code>Record</code> structure in bytes (64 …\nComputes the segment size for the partition table image.\nComputes the signature for the partition table image.\nCreates a signature for the partition table image using …\nA flag that indicates whether debugging should be skipped …\nReturns a default <code>TrapConfig</code> with all fields set to 0 or …\nReturns a default <code>Record</code> with zeroed and invalid fields.\nReturns a default <code>PartitionTableImage</code> with default values …\nReturns the argument unchanged.\nConverts a 16-bit integer to a <code>TrapConfig</code> by unpacking the …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns a reference to the 32-byte hash value of the …\nReturns a reference to the <code>hash_key</code>.\nReturns the record for a specific partition type.\nReturns the record for a specific partition type.\nReturns the records in the partition table.\nReturns the user binary data.\nReturns the user extension data (12 bytes).\nChecks if a record exists for a specific partition type.\nChecks whether the <code>hash_key</code> is valid.\nCalls <code>U::from(self)</code>.\nConverts a <code>TrapConfig</code> back into a 16-bit integer by …\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nThe length of the partition in bytes (4 bytes).\nThe level of the trap (1 bit, 0 or 1).\nCreates and returns a new record for a specific partition …\nThe partition type (1 byte). This is an enum (<code>PartitionType</code>…\nThe pin number (5 bits, value range: 0-31).\nThe port number (3 bits, value range: 0-7).\nParses a <code>Record</code> from a binary stream.\nParses a <code>PartTab</code> from a binary stream.\nParses a <code>PartitionTableImage</code> from a binary stream.\nRemoves a partition record from the partition table.\nSets the <code>hash_key</code> to a new value.\nSets the segment size for the partition table image.\nSets the signature for the partition table image.\nSets the user binary data in the partition table.\nSets the user extension data in the partition table.\nThe starting address of the partition in the firmware …\nWhether the trap configuration is valid (1 bit).\nWrites a <code>Record</code> to a binary stream.\nWrites a <code>PartTab</code> to a binary stream.\nWrites a <code>PartitionTableImage</code> to a binary stream.\nRepresents a section in a sub-image.\nComputes the aligned length of the section data, ensuring …\nComputes the aligned size of the section, including the …\nReturns a default <code>Section</code> with default headers and an …\nThe header that defines the entry point and loading …\nReturns the argument unchanged.\nReturns a reference to the section’s data.\nThe metadata and configuration for the section.\nCalls <code>U::from(self)</code>.\nCreates a new <code>Section</code> with a specified data capacity.\nReads a <code>Section</code> from a stream.\nReplaces the current content of the <code>data</code> field with a new …\nWrites a <code>Section</code> to a stream.\nRepresents information about the flash memory …\nRepresents the configuration for forcing the use of an old …\nRepresents the SPI (Serial Peripheral Interface) …\nRepresents system data related to the device, including …\nReturns a default <code>ForceOldImage</code> configuration with all …\nReturns the default <code>SpiConfig</code> with default I/O mode and …\nReturns the default <code>FlashInfo</code> instance.\nReturns the default <code>SystemData</code> instance with preset values.\nInformation about the flash memory, including ID and size.\nReturns the argument unchanged.\nConverts a 32-bit unsigned integer into a <code>ForceOldImage</code> …\nReturns the argument unchanged.\nConverts a 32-bit unsigned integer into a <code>SpiConfig</code> …\nConverts a 32-bit unsigned integer into a <code>FlashInfo</code> …\nReturns the argument unchanged.\nReturns the argument unchanged.\nRetrieves the Bluetooth parameter data as a reference.\nRetrieves the SPI calibration configuration as a reference.\nConverts a <code>ForceOldImage</code> instance into a 32-bit unsigned …\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nConverts a <code>SpiConfig</code> instance into a 32-bit unsigned …\nCalls <code>U::from(self)</code>.\nConverts a <code>FlashInfo</code> instance into a 32-bit unsigned …\nCalls <code>U::from(self)</code>.\nChecks whether the old image configuration is active.\nCreates a new instance of <code>ForceOldImage</code> with the specified …\nConfiguration for forcing the usage of an older image, …\nAddress of the OTA2 image, or <code>None</code> if OTA1 is active.\nSize of the OTA2 image, or <code>None</code> if OTA1 is active.\nRetrieves the pin number of the configuration.\nRetrieves the port number of the configuration.\nReads <code>SystemData</code> from a stream (e.g., file, memory, or …\nSets the Bluetooth parameter data.\nSets the SPI calibration configuration.\nConfiguration for the SPI bus, including IO mode and speed.\nBaud rate for the UART logging interface.\nWrites <code>SystemData</code> to a stream (e.g., file, memory, or …\nComputes an HMAC-MD5 signature for the provided key and …\nComputes an HMAC-SHA256 signature for the provided key and …\nComputes an MD5 hash of the provided data.\nComputes a SHA-256 hash of the provided data.\nSkips bytes in the provided reader to ensure that the next …\nWrites the specified byte <code>fill</code> repeatedly to the writer …")