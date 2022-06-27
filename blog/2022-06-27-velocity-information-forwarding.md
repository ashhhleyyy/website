# Velocity modern player information forwarding

When working on [the fallback server for Nucleoid](https://github.com/NucleoidMC/fallblock), one of the required features was to implement Velocity's [modern player information forwarding](https://velocitypowered.com/wiki/users/forwarding/), to allow skins to correctly load in on the fallback server. To do this, I first needed to find information about how the protocol worked, and I struggled to find any official documentation, and so I've summarised my understanding of how it works here.

## Initial connection

When the Minecraft client connects to a server, it begins in the handshake state, and immediately sends a packet to the server, specifying its protocol version and the next state it would like to switch to, which can be either status or login. For the purposes of implementing player information forwarding, the status state is not important.

## Login phase

When connecting to an online-mode server, the login phase usually involves some cryptography and exchanges with Mojang's servers in order to authenticate the user and enable encryption of the protocol, and this is the point where information such as skin data is loaded by the server. However, when using a proxy like Velocity, the connection between the proxy and the backend server is not encrypted, and the connection is made as if it is to an offline-mode server, which does not involve any communication with Mojang, as the proxy has already performed these steps with the client. This means that the backend server has no access to the player data that velocity obtained, and so a protocol for player information forwarding is required.

## Modern player information forwarding

Velocity implements two methods for forwarding player information, the legacy BungeeCord protocol (which is considered insecure), and the newer 'modern forwarding'.

Modern information forwarding makes use of the [Login Plugin Request](https://wiki.vg/Protocol#Login_Plugin_Request)/[Response](https://wiki.vg/Protocol#Login_Plugin_Response) packets during the login phase in order to send the data in a vanilla-compatible way.

To perform modern information forwarding, the backend server must send a Login Plugin Request packet before the Login Success packet, with the channel set to `velocity:player_info`, and the following payload:

| Field | Type | Value |
| ----- | ---- | ----- |
| Protocol version | `byte` | `1` for `MODERN_FORWARDING_DEFAULT` (up to MC 1.18), `2` for `MODERN_FORWARDING_WITH_KEY` (MC 1.19+) |

The velocity proxy should then respond with a corresponding Login Plugin Response, which will contain the player information.

The structure of the packet is the following:

| Field | Type | Description |
| ----- | ---- | ----------- |
| Signature | `byte[32]` | See the Validation section below |
| Version | `varint` | The version of the forwarding protocol the server is using. See above |
| Client Address | `String[32767]` | The IP address of the connecting client, in textual representation |
| Player ID |  `UUID` | The Mojang UUID of the connecting player |
| Username | `String[16]` | The Mojang username of the connecting player |
| Properties | `Property[]` | A length (as a `varint`) prefixed array of `Properties` (see below) |

If the returned version is `2` (`MODERN_FORWARDING_WITH_KEY`), the following fields are added to the response:

| Field | Type | Description |
| ----- | ---- | ----------- |
| Public key expiry | `long` | The expiry of the players chat signing key |
| Public key | `byte[512]` | The encoded RSA public key used by the client to sign messages |
| Public key signature | `byte[4096]` | The Mojang-provided signature of the key |

### Properties

Each element in the properties array is a structure like this:

| Field | Type | Description |
| ----- | ---- | ----------- |
| Name | `String[32767]` | The name of the property |
| Value | `String[32767]` | The value of the property |
| Has signature | `boolean` | Whether or not a signature follows |
| Signature | `String[32767]` | The signature for this property |

These fields correspond to the way texture data is returned from [the Mojang API](https://wiki.vg/Mojang_API#UUID_to_Profile_and_Skin.2FCape) during regular auth.

### Validation

One of the major improvements of modern information forwarding is that the data is signed using a key shared between the proxy and backend server, which prevents somebody from impersonating the proxy if the server is accidentally exposed to the public internet.

The signature is implemented using a [HMAC](https://en.wikipedia.org/wiki/HMAC) with [SHA256](https://en.wikipedia.org/wiki/SHA256) as the hash function. It is calculated from the remaining content of the payload after the signature field, and if it is invalid, then the client should be disconnected immediately.

## Thanks

These notes are mostly based on study of the code for the [FabricProxy-Lite](https://github.com/OKTW-Network/FabricProxy-Lite) mod, along with some cross checking with the implementation in the actual Velocity code.
