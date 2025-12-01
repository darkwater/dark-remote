# dark remote

like the iOS apple tv remote app but for linux and actually useful and customizable

<img width="590" height="1280" alt="image" src="https://github.com/user-attachments/assets/b097d421-a3d3-4b36-b946-d44c26b4e767" />

still cooking

more docs later probably but i'm just gonna tell you how hdmi-cec works

## HDMI-CEC

The cec module (or whatever the terminology is gonna be) needs some setup to
work. Most consumer GPUs don't support CEC. Some notable exceptions:

(off the top of my head, open an issue if i'm wrong)

- Raspberry Pi
- Steam Deck using the official dock I believe?
- Steam Machine supposedly will support it
- Some Intel NUCs?

If your GPU supports CEC I'm assuming you'll already have a `/dev/cec0` so just
check if that device exists to see if your hardware supports it.

For anything else, Pulse-Eight sells [a USB
adapter](https://www.pulse-eight.com/p/104/usb-hdmi-cec-adapter) that you pipe
your HDMI connection through, and exposes a serial device to work with CEC.

You can use `inputattach(1)` to turn that into a `/dev/cec0`, by doing
something like:

```bash
$ sudo inputattach --pulse8-cec "/dev/serial/by-id/usb-Pulse-Eight_CEC_Adapter_v12-if00"
```

On Arch Linux, `inputattach(1)` is provided by the `linuxconsole` package. This
package also comes with a `systemd` service that's set up such that you can
edit `/etc/conf.d/inputattach` like so:

```bash
IAPARAMS=(
  "--pulse8-cec /dev/serial/by-id/usb-Pulse-Eight_CEC_Adapter_v12-if00"
)
```

And then just enable the service:

```bash
$ sudo systemctl enable --now inputattach
```

You should have a `/dev/cec0` now.

You can use `cec-ctl(1)` to play around with it, provided by `v4l-utils` on Arch:

```bash
$ cec-ctl --to 0 --image-view-on # Tell 0 (the TV) to turn on its image view (display)

$ cec-ctl --to 0 --standby

$ sudo cec-ctl --monitor-all
```

To change inputs, we'll need to know our physical address. The TV can
communicate this using EDID, the same stuff it uses to communicate possible
resolutions, refresh rates, model name, etc. If your GPU supports CEC it might
already be configured, but the Pulse-Eight adapter has no way of knowing which
connected output corresponds to the connection that's piped through it.
Strictly speaking, it could even be connected to a different machine!

Luckily, you probably know what output it is, and telling the adapter is
simple. You should have a bunch of entries in `/sys/class/drm` corresponding to
your video outputs, eg. `card1-HDMI-A-1`. The EDID should be available in
there, and you can simply do:

```bash
$ cec-ctl --phys-addr-from-edid "/sys/class/drm/card1-HDMI-A-1/edid"
```

Running `cec-ctl` will now show a physical address, eg. `2.0.0.0` (which might
correspond to "HDMI 2"). You can then do:

```bash
$ cec-ctl --active-source phys-addr=2.0.0.0
```

Additionally, for certain commands you need to assign a logical address. You
can simply do that with something like:

```bash
$ cec-ctl --playback
```
