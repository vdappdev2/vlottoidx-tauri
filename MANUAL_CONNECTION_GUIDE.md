# Manual RPC Connection Guide

Connect VerusIDX to a Verus daemon running on another computer.

---

## Quick Start

### 1. Find Your Daemon's Information

**On the computer running Verus daemon:**

#### macOS
```bash
# Get IP address
ifconfig | grep "inet " | grep -v 127.0.0.1
```
Look for IP starting with `192.168.` or `10.0.`

**Config location:** `~/Library/Application Support/Komodo/vrsctest/vrsctest.conf`

#### Windows
1. Settings → Network & Internet → Properties
2. Find **IPv4 Address** (e.g., `192.168.1.105`)

**Config location:** `C:\Users\YourUsername\AppData\Roaming\Komodo\vrsctest\vrsctest.conf`

#### Linux
```bash
# Get IP address
ip addr show | grep "inet "
```
Look for IP starting with `192.168.` or `10.0.`

**Config location:** `~/.komodo/vrsctest/vrsctest.conf`

---

### 2. Configure Daemon for Remote Access

**Edit the config file** (vrsctest.conf or VRSC.conf):

```
rpchost=0.0.0.0
rpcallowip=0.0.0.0/0
rpcuser=your_username
rpcpassword=your_password
rpcport=18843
```

**Restart the daemon** after saving changes.

---

### 3. Open Firewall Port

#### macOS
System Settings → Network → Firewall → Options → Add port 18843

#### Windows
1. Start → "Windows Defender Firewall" → Advanced Settings
2. Inbound Rules → New Rule → Port
3. TCP, port `18843` → Allow the connection
4. Check: Domain, Private, Public → Name: "Verus RPC"

**Network profile:** Settings → Network → Set to **Private** (not Public)

#### Linux (Ubuntu/Debian)
```bash
sudo ufw allow 18843/tcp
```

---

### 4. Connect in VerusIDX

On the computer running VerusIDX:

1. Launch app → Show manual connection
2. Enter:
   - **Host:** Daemon computer's IP (e.g., `192.168.1.105`)
   - **Port:** `18843` (testnet) or `27486` (mainnet)
   - **Username:** Value after `rpcuser=` in config
   - **Password:** Value after `rpcpassword=` in config
3. Click "Connect Remotely"

---

## Security Considerations

### ⚠️ Important Security Notes

**Unencrypted Connection:**
- Manual RPC uses HTTP (not HTTPS)
- Credentials and data sent in **plain text**
- Anyone on the same network can intercept traffic

**Safe Use Cases:**
- ✅ Same trusted home network
- ✅ Through SSH tunnel (see below)
- ✅ Through VPN
- ❌ Public Wi-Fi / untrusted networks

### Remote Warning

When connecting to a non-localhost daemon, the app will show a security warning. Only proceed if:
- You own both computers
- You're on a trusted private network
- You're using an SSH tunnel or VPN

---

## Advanced: SSH Tunnel (Encrypted)

For maximum security, use an SSH tunnel to encrypt all RPC traffic.

### Setup

#### 1. Enable SSH Server (Daemon Computer)

**macOS:**
System Settings → Sharing → Enable "Remote Login"

**Windows:**
Settings → Apps → Optional Features → Add "OpenSSH Server"
Services → Start "OpenSSH SSH Server"

**Linux:**
```bash
sudo apt install openssh-server
sudo systemctl start ssh
```

#### 2. Create Tunnel (VerusIDX Computer)

**macOS/Linux:**
```bash
ssh -L 18843:localhost:18843 username@daemon-ip-address
```

**Windows PowerShell:**
```powershell
ssh -L 18843:localhost:18843 username@daemon-ip-address
```

Replace:
- `username` = Your account name on daemon computer
- `daemon-ip-address` = Daemon's IP (e.g., `192.168.1.105`)

**Keep terminal window open while using VerusIDX**

#### 3. Connect in App

- **Host:** `127.0.0.1` (localhost - traffic goes through encrypted tunnel)
- **Port:** `18843`
- **Username/Password:** From Verus config

---

## Troubleshooting

### "Connection test failed"

**Check daemon is running:**
```bash
# macOS/Linux
ps aux | grep verus

# Windows PowerShell
Get-Process | Select-String verus
```

**Test RPC manually:**
```bash
curl -u username:password --data-binary '{"jsonrpc":"1.0","method":"getinfo","params":[]}' http://daemon-ip:18843
```

**Common issues:**
- Firewall blocking port 18843
- `rpchost` still set to `127.0.0.1` (change to `0.0.0.0`)
- Wrong IP address (use `ipconfig`/`ifconfig` to verify)
- Daemon not restarted after config changes

### "Daemon offline or unreachable"

**Verify port is open:**
```bash
# macOS/Linux
nc -zv daemon-ip 18843

# Windows PowerShell
Test-NetConnection -ComputerName daemon-ip -Port 18843
```

If this fails, firewall is blocking the connection.

---

## Best Practices

1. **Use strong passwords** in RPC config
2. **Don't remember credentials** for remote connections unless it's your own server
3. **Use SSH tunnel** when connecting over internet or untrusted networks
4. **Set network to Private** on Windows (not Public)
5. **Restart daemon** after any config changes
6. **Close port 18843** in firewall when not in use for added security

---

## FAQ

**Q: Can I connect over the internet?**
A: Yes, but you **must** use an SSH tunnel or VPN. Never expose RPC directly to the internet.

**Q: What's the difference between localhost and remote?**
A: Localhost (`127.0.0.1`) connects to daemon on same computer. Remote connects to different computer using its IP address.

**Q: Do I need to open ports for localhost connections?**
A: No, firewall rules only apply to remote connections.

**Q: Can multiple VerusIDX instances connect to one daemon?**
A: Yes, one daemon can handle multiple RPC connections simultaneously.

**Q: Why does the app say "Connect Remotely" even for localhost?**
A: The manual connection form handles both local and remote connections. Button text is simplified for clarity.

---

## Support

For issues or questions:
- GitHub: https://github.com/VerusCoin/verusidx/issues
- Discord: Verus Community

**Remember:** Manual connection with proper security (SSH tunnel or trusted private network) enables powerful remote access to your Verus daemon!
