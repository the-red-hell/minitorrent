# Your BitTorrent-Client for embedded systems

After implementing a simple [BitTorrent-Client using Rust with standard library](https://github.com/paulhkz/bittorrent-rust.git), I still had an ESP32C3 laying around.
So the next logical thing to do is: implementing a BitTorrent-Client on that thing.

## Hardware-Stack

- [ESP32C3-Supermini](https://www.espboards.dev/esp32/esp32-c3-super-mini-plus/)
- SD-Card-Reader via SPI ([something like this](https://www.amazon.com/HiLetgo-Adater-Interface-Conversion-Arduino/dp/B07BJ2P6X6))
- a capacitor (works better with than without, Gemini told me): specifically a 10µF one between VCC and 5V

In the following picture you can see the components:
![Hardware](https://github.com/user-attachments/assets/f64c130d-118f-47b2-8a67-9908ddafe92b)
(top left: SD-Card-Reader with SD-Card, top right: ESP32C3-Supermini, bottom: 10µF capacitor)

### Pinout

| ESP32C3 Pin | SPI Function | SD Card Module Pin |
|:-----------:|:------------:|:------------------:|
| GPIO4       | SCK          | CLK                |
| GPIO5       | MISO         | DO (MISO)          |
| GPIO6       | MOSI         | DI (MOSI)          |
| GPIO7       | CS           | CS                 |
| 5V          | —            | VCC                |
| GND         | —            | GND                |

> **Note:** A 10µF capacitor is placed between VCC and 5V for stability.

```
ESP32C3-Supermini          SD Card Module
┌──────────┐               ┌──────────┐
│      GPIO4 ──── SCK ────── CLK      │
│      GPIO5 ──── MISO ───── DO       │
│      GPIO6 ──── MOSI ───── DI       │
│      GPIO7 ──── CS ─────── CS       │
│          5V ──┬─────────── VCC      │
│               ┊ (10µF cap)          │
│         GND ──┴─────────── GND      │
└──────────┘               └──────────┘
```


## Project-Structure

- *bencode*: A simple library for de- and encoding stuff in the [Bencode](https://en.wikipedia.org/wiki/Bencode)-Style. I found no no-std implementation so I made my own
- *core-logic*: The abstraction which essentially sits above the HAL. It doesn't use any hardware-specific stuff and my goal is to make it generic over other micro-controllers.
- *esp-app*: Hardware-specific implementations for the Filesystem and Wifi. My goal is to keep that as small as possible.

## Flow

(The ticked entries are implemented.)

- [x] we open a directory called `torrent`
- [x] in there we find the first file ending with `.torrent`
*the rest is basically just the normal bittorrent protocol*
- [x] parse the file
- [x] request the tracker via the provided `announce`-key
- [x] receive the response & parse it into a `TrackerResponse`
- [x] download the stuff if everything works correctly
- [ ] the rest (handle errors, seed)...
