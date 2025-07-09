# nss-userhosts

Name Service Switch (`nss(5)`) plugin for user-configurable hosts lookup (`hosts(5)`).

## Installation

0. Build:  
	`cargo build --release`

1. Copy `libnss_userhosts.so` to `/lib/x86_64-linux-gnu/`:  
	`sudo cp target/release/libnss_userhosts.so /lib/x86_64-linux-gnu/libnss_userhosts.so`)

2. Notify `ldconfig(8)` of the change and add the .so.2 symlink:  
	`sudo ldconfig -n /lib/x86_64-linux-gnu`

3. Add `userhosts` to the hosts line in `nsswitch.conf(5)`:  
	`hosts:          userhosts files mdns4_minimal [NOTFOUND=return] dns`

## Usage

Create and populate `$HOME/hosts` as you would `/etc/hosts`. Alternatively, set `USERHOSTS_FILE` to the path of a file you would like to use instead of `$HOME/hosts`.

The environment variable `USERHOSTS` can also be set to introduce `hosts(5)` entries. In this case, split multiple entries with a `;` (semicolon):
```
$ USERHOSTS="127.1.1.1 one; 127.2.2.2 two" getent hosts one two
127.1.1.1       one
127.2.2.2       two
```
