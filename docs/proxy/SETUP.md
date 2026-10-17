# Filter Proxy - Permissions and Setup

`--filter` grabs the physical touchscreen (`EVIOCGRAB`) and re-emits its
event stream on a virtual uinput clone with tangible contacts suppressed.
Both operations need device permissions; root is not required.

## Required permissions

| Resource | Device node | Access |
|----------|-------------|--------|
| Physical touchscreen | `/dev/input/eventN` | read + write (grab) |
| Virtual clone | `/dev/uinput` | read + write |

## User groups (quick setup)

Most distros ship `/dev/uinput` with group `uinput` and `event*` nodes
with group `input`:

```bash
sudo usermod -aG input,uinput $USER
# re-login or: newgrp input
```

If your distro has no `uinput` group, create it together with the udev
rule below.

## udev rule (persistent, recommended)

Create the `uinput` group and assign the node:

```bash
sudo groupadd -f uinput
sudo usermod -aG uinput,input $USER
```

`/etc/udev/rules.d/99-tangible-objects.rules`:

```udev
# uinput for the virtual touchscreen clone
KERNEL=="uinput", GROUP="uinput", MODE="0660"

# Touchscreens readable/grabbed by the input group (usually the default)
SUBSYSTEM=="input", KERNEL=="event[0-9]*", GROUP="input", MODE="0660"
```

Reload:

```bash
sudo udevadm control --reload-rules
sudo udevadm trigger
```

## Verify

```bash
ls -l /dev/uinput /dev/input/event4
id | grep -o 'input\|uinput'
```

Expected: `crw-rw---- root uinput` on `/dev/uinput`, `root input` on the
event node, and both groups listed for your user.

## Kernel module

`uinput` is usually loaded automatically. If `/dev/uinput` is missing:

```bash
sudo modprobe uinput
echo uinput | sudo tee /etc/modules-load.d/uinput.conf
```

## systemd service (optional)

The proxy daemon runs fine as a normal user once the permissions above
are in place. For a service unit, set `Group=`/`SupplementaryGroups=` to
`input uinput` instead of running as root.
