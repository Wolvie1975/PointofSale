# Deploying the kiosk to the office machine

Turns a Fedora machine into a single-purpose kiosk: it boots straight into the Healing Hideaway app under `cage`, with no desktop and no way out from the keyboard or touchscreen.

> **Warning:** This is still a POC. Bookings live **in memory**, so they're lost whenever the app restarts or the machine reboots. Don't take real client bookings on it yet.

## 1. Build the package (on the dev machine)

```bash
cd kiosk
npm run build:kiosk
```

This builds with the kiosk window settings (`src-tauri/tauri.kiosk.conf.json`: full screen at 1920×1080, no title bar, no resizing). Plain `npm run tauri dev` keeps a normal window for development.

The package lands in `target/release/bundle/rpm/` at the repo root:

```
Healing Hideaway-0.1.0-1.x86_64.rpm
```

The file name has a space in it, so keep the quotes in the commands below. It installs `/usr/bin/healing-hideaway-kiosk` and pulls in `cage`, GTK and WebKit.

## Trying it on your own Fedora machine

Use this to test a build on an everyday computer **without** turning it into a kiosk. Run the commands in a terminal from the repo root (the folder with `kiosk/` and `target/` in it):

```bash
cd ~/Development/Environment/Rust_Projects/PointofSale

# First install
sudo dnf install "./target/release/bundle/rpm/Healing Hideaway-0.1.0-1.x86_64.rpm"

# A newer build with the same version number (dnf would say "already installed")
sudo dnf reinstall "./target/release/bundle/rpm/Healing Hideaway-0.1.0-1.x86_64.rpm"
```

Then start it, from any folder:

```bash
HH_STAFF_PIN=2468 healing-hideaway-kiosk
```

- It opens full screen with no title bar. **Alt+F4** closes it.
- **Staff close:** hold **Healing Hideaway** in the top bar for 3 seconds (or press Ctrl+Alt+S) and enter the PIN you started it with (any 4–8 digits; `2468` above). Without `HH_STAFF_PIN`, the pad says no PIN is set up and won't close.
- Opening it from the app menu also works, but then there's no PIN, so use Alt+F4 to close.

To try it exactly as the kiosk runs it, press Ctrl+Alt+F3, log in, and run:

```bash
HH_STAFF_PIN=2468 cage -s -- healing-hideaway-kiosk
```

Ctrl+Alt+F2 gets you back to your desktop. If the window is blank, put `WEBKIT_DISABLE_DMABUF_RENDERER=1` at the start of the command.

To remove it:

```bash
sudo dnf remove healing-hideaway
```

> **Warning:** Don't do step 4 below on an everyday machine. It turns off the normal desktop login and boots straight into the kiosk. Steps 2–5 are for the dedicated office machine only.

## 2. Before you lock the machine down

Once the kiosk service is enabled, the keyboard can't reach a desktop or console. Make sure you have another way in **first**:

```bash
sudo systemctl enable --now sshd
ip -4 addr            # note the machine's address
```

From another computer, check that `ssh <user>@<address>` works before going on.

## 3. Install and try it by hand (on the office machine)

Copy the `.rpm` and `deploy/kiosk.service` to the office machine, then:

```bash
sudo dnf install "./Healing Hideaway-0.1.0-1.x86_64.rpm"
```

Try it under `cage` from a text console. Press Ctrl+Alt+F3, log in, then run:

```bash
cage -s -- healing-hideaway-kiosk
```

To try the staff close button here, start it with a PIN: `HH_STAFF_PIN=2468 cage -s -- healing-hideaway-kiosk`. `-s` lets you get back out with Ctrl+Alt+F2 while testing. The boot service in step 4 leaves it off. If the window is blank, run `WEBKIT_DISABLE_DMABUF_RENDERER=1 cage -s -- healing-hideaway-kiosk` instead (the service already sets this).

## 4. Start it at boot

```bash
sudo useradd -m kiosk
sudo cp kiosk.service /etc/systemd/system/kiosk.service
sudo systemctl daemon-reload
sudo systemctl disable gdm             # the kiosk replaces the desktop login
sudo systemctl enable kiosk.service
sudo reboot
```

The machine should boot straight into the app. If the app ever exits, systemd starts it again within 2 seconds.

### Set the staff PIN

Staff can close the app from the screen: press and hold **Healing Hideaway** in the top bar for 3 seconds (or press Ctrl+Alt+S), then enter the staff PIN. The PIN is 4–8 digits. It's kept out of the code and the repo, in a systemd override that only root can read:

```bash
sudo mkdir -p /etc/systemd/system/kiosk.service.d
printf '[Service]\nEnvironment=HH_STAFF_PIN=%s\n' '<your PIN>' | sudo tee /etc/systemd/system/kiosk.service.d/staff-pin.conf >/dev/null
sudo chmod 600 /etc/systemd/system/kiosk.service.d/staff-pin.conf
sudo systemctl daemon-reload
sudo systemctl restart kiosk
```

Without a PIN, the app can't be closed from the screen at all. After 5 wrong PINs it refuses every attempt for 60 seconds.

> **Note:** Under this service, **Close app restarts the app fresh** after 2 seconds. That clears the screen and all in-memory bookings. To actually stop the kiosk, use SSH (see Maintenance).

## 5. Check it

These are checklist item #1 and the touch check from the guide (section 9):

- [ ] It boots to Home, full screen at 1920×1080, with no title bar, panel or desktop
- [ ] Ctrl+Alt+F2 through F6, Alt+Tab and Alt+F4 do nothing (Alt+F4 at most restarts the app)
- [ ] Right-click or long-press shows no menu
- [ ] **Touch only:** finish a booking using just the touchscreen
- [ ] It returns to Home after 90 seconds idle

## Maintenance (over SSH)

```bash
sudo systemctl stop kiosk              # stop the app
sudo systemctl restart kiosk           # restart it (clears all bookings)
journalctl -u kiosk -b                 # logs since boot
```

Update to a new build:

```bash
sudo dnf install "./Healing Hideaway-<version>-1.x86_64.rpm"
sudo systemctl restart kiosk
```

Undo the kiosk setup and get the normal desktop back:

```bash
sudo systemctl disable --now kiosk
sudo systemctl enable gdm
sudo reboot
```
