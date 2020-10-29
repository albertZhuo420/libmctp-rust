# libMCTP

libMCTP is a MCTP (Management Component Transport Protocol) implementation
for Rust.

libMCTP aims to implement the MCTP protocol as described in the DMTF DSP2016
specification, which can be found here:
    https://www.dmtf.org/sites/default/files/standards/documents/DSP2016.pdf

MCTP allows multiple transport layers, the protocols supported by this library
include:
 * SMBus/I2C version 1.2.0. See DMTF DSP0237 (https://www.dmtf.org/sites/default/files/standards/documents/DSP0237_1.2.0.pdf)