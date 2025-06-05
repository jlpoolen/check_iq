# check_iq

**check_iq** is a high-performance Rust utility for analyzing RTL-SDR `.iq` ( aka "IQ" or "I/Q", In-phase & Quadrature) binary capture files to detect **signal clipping** in I/Q samples. 

Clipping consists of saturated I/Q values at 0 or 255. It typically results from overly strong RF signals — such as aircraft flying directly overhead and transmitting in the same specified frequency — and is a key diagnostic when evaluating gain and antenna performance.

## Features

- Detects **clipped I/Q values** (`0` and `255`)
- Calculates **clipping percentage** across all samples
- Reports **per-second clipped sample counts**
- Supports **custom sample rates**
- UTC and **localtime output support**
- **Fast, targeted second-based analysis** via `--break_out`
- Optimized for **multi-gigabyte** `.iq` files

## Requirements

- [Rust](https://www.rust-lang.org/tools/install)

## Build

```bash
cargo build --release
```

Resulting binary will be:

```bash
target/release/check_iq
```

## Usage

```bash
check_iq <filename> <sample_rate> [--epoch_UTC <value>] [--output_localtime true] [--break_out <second_number>]
```


- `<filename>` – Path to `.iq` file from RTL-SDR.
- `<sample_rate>` – Sample rate used during capture (e.g., 2400000).
- `--epoch_UTC <int>` – (Optional) Starting UTC epoch time for correlating clipping time.
- `--output_localtime true` – (Optional) Display output in system's localtime (respects DST). Requires `--epoch_UTC`.
- `--break_out <second>` – (Optional) Displays detailed clipping counts for one specific second only (much faster than full scan).
- 
## Examples

### Basic Full Scan

```sh
check_iq /tmp/file.iq 2400000
```

Output:

```
File: /tmp/file.iq
Total I/Q pairs processed: 8640004096
--- Clipping Statistics ---
I = 0     :    2926263
I = 255   :    2912799
Q = 0     :    2959585
Q = 255   :    2918511
Clipping percentage: 0.067808%

--- Clipping per second ---
second    143: 10448 clipped samples
second    144: 1064928 clipped samples
...
```

### UTC Timestamp Conversion

```sh
check_iq /tmp/file.iq 2400000 --epoch_UTC 1748797200
```

Output:

```
File: /tmp/file.iq
Sunday, June  1, 2025 at 17:00 UTC
...
--- Clipping per second ---
1748797343: 10448 clipped samples
1748797344: 1064928 clipped samples
...
```

### Local Time Output

```sh
check_iq /tmp/file.iq 2400000 --epoch_UTC 1748797200 --output_localtime true
```

Output:

```
File: /tmp/file.iq
Sunday, June  1, 2025 at 10:00 AM -07:00
...
--- Clipping per second ---
10:02:23: 10448 clipped samples
10:02:24: 1064928 clipped samples
...
```

### Peek into a Specific Second

```sh
check_iq /tmp/file.iq 2400000 --break_out 1296
```

Output:
```
--- Detailed Clipping for Second 1296 ---
second   1296: 1233834 clipped samples
         I: 0:310655 255:308881
         Q: 0:305503 255:308795
```

---

## Error Handling

- Missing sample rate → error
- `--output_localtime` requires `--epoch_UTC`
- `--break_out` must reference a valid second (within duration of the file)
- Typos like `--epoch_UCT` will trigger immediate error
- Unrecognized options produce explicit failure

---
## Background

This tool is useful when analyzing SDR recordings for saturation events due to close-range transmissions, especially aircraft transmitting near your antenna. For example:

- Aircraft passes overhead at 200–300 feet
- Your gain is too high
- `.iq` capture shows thousands or millions of clipped samples
- These can now be quantified and time-mapped

See:  
https://ham.stackexchange.com/questions/16457/what-is-iq-in-the-context-of-sdrs

---
## Suggested Workflow

1. Run `check_iq` on `.iq` file
2. Identify time(s) with heavy clipping
3. Use `--break_out` to get fine-grained insight into individual seconds
4. Cross-reference timing with aircraft logs or ADS-B data using `epoch_UTC`

---

## Notes

- `.iq` files are raw unsigned 8-bit, interleaved I/Q
- Clipping: 0 or 255 = bad, may distort demodulation
- With a Ryzen 7950, the tool processes 17GB in ~90–120 seconds (release mode)

---
## License

MIT License

---

Created by ChatGPT +  John Laurence Poole, 2025-06-04
