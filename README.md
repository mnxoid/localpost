# Localpost

This is a tool that allows sending files between machines on the same LAN in a rather ergonomic manner.

## Windows

Since Windows firewall is a pain to work with, in order for the tool to work you need to punch a hole in it:

```shell
New-NetFirewallRule `
    -DisplayName "Localpost" `
    -Direction Inbound `
    -Action Allow `
    -Protocol TCP `
    -LocalPort 9057
```