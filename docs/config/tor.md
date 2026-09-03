# Configure Tor Proxy for Onion-Service Relays

> `n34 config tor`

**Usage:**
```
Enable tor proxy for onion-service relays

Usage: n34 config tor [OPTIONS] [PORT]

Arguments:
  [PORT]  Tor proxy port. Set to 0 to disable tor proxy [default: 9050]
```

Configure proxying of onion-service relays through your local Tor proxy. This
setting allows n34 to connect to onion relays that are only accessible through
the Tor network.

- **Default port**: If you run `n34 config tor` without specifying a port, it
  defaults to `9050` (Tor's default SOCKS port).
- **Disable proxying**: Set the port to `0` to disable onion-service relay
  proxying entirely. Use `n34 config tor 0`.
- **Custom ports**: If your Tor proxy runs on a different port, specify it
  directly, e.g., `n34 config tor 9150`.

**Note**: This only affects connections to onion-service relays. Regular relay
connections continue to use your standard network connection.
