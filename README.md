## reduce-target
this lists and optionally removes all target directories from a top-level down

this is useful for, **dumbly**, nuking a ton of rust projects build caches to restore a few (dozen ..or hundreds in my case) gigabytes of disk space

it only recursively transverses until it finds a directory named `target`
usage:
```txt
Optional arguments:
  -h, --help                 show this message
  -d, --directory DIRECTORY  root directory to search
  -s, --stats                prints directories statistics
  --sweep                    sweeps all directories
```

if `-d` is not provided, then the `CWD` (the . directory) is assumed

for example, removing all target directories under ~/dev/rs:

```
reduce-target -d ~/dev/rs -s --sweep
               ^           ^   ^--- remove directories
               |           \--- display stats
               \--- root directory
```
