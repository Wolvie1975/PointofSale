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

`-s` lets you get back out with Ctrl+Alt+F2 while testing. The boot service in step 4 leaves it off. If the window is blank, run `WEBKIT_DISABLE_DMABUF_RENDERER=1 cage -s -- healing-hideaway-kiosk` instead (the service already sets this).

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
