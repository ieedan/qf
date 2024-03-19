# qf
qf (Quick-Find) is a file search CLI that makes files searching speedy and powerful.

```bash
C:\Users\aidan\Documents> qf *.rs --a[github] --i[rustlings,package,node_modules] --ra
Searching for *.rs...
Ignoring ["rustlings", "package", "node_modules"]
File: main.rs
Path: C:\Users\aidan\Documents\GitHub\ips\src\main.rs
File: main.rs
Path: C:\Users\aidan\Documents\GitHub\qf\src\main.rs
Searched 29901 files. Found 2 files.
Completed in 624.45ms
```

### Basic Use
``` bash
# Find all files that match 'index.js'
qf index.js
```

### Wildcards
**qf** allows for the use of wild cards to improve your search.
```bash
qf index.js # Find all files that match index.js 

qf *.js # Find all files that end with '.js'

qf index* # Find all files that start with 'index'

qf *index* # Find all files that contain 'index'
```

### Flags
- `--i[args]` Ignore directories. [Docs](#iargs)
- `--ri` Only apply ignore flag to cwd (Current Working Directory). [Docs](#ri)
- `--a[args]` Allow directories. [Docs](#aargs)
- `--ra` Only apply allow flag to cwd (Current Working Directory). [Docs](#ra)
- `--c[arg]` Allows you to specify the minimum entries in a directory before allowing concurrency. [Docs](#carg)
- `--dc` Allows you to disable concurrency. [Docs](#dc)

#### `--i[args]`
This allows you to skip searching files in the provided subdirectories. That looks something like this. 

Usage:
```bash
# Ignore folders that match 'node_modules'
qf index.js --i[node_modules]

# For folder names including a space you must wrap them in '' or ""
qf index.js --i['your name']
```

#### `--ri`
This flag effects where the `--i` flag applies. When the `--ri` flag is provided the `--i` flag only applies to the cwd.

Lets say your file structure looks like this:
```text
root
├── modules
├── src
├── release
|   └── random.exe
└── target
    ├── debug
    |   └── qf.exe
    └── release
        └── qf.exe
```

Here we may want to search for a `*.exe` file but running `qf *.exe` will return all executables under the cwd. What if you want to ignore the release directory at the root but not any sub-folders?

Again this is the utility of the `--r` flag. 

```bash
qf *.exe --i[release] --ri
```

This will ignore the `release` directory but only at the cwd. Any sub-folders will still be searched.

#### `--a[args]`
This allows you to only search directories that match the provided directories.

Usage:
```bash
# Only searches directories that match 'src'
qf index.js --a[src]
```

This can also be useful if you just want to list the files in your cwd (Current working directory) without searching all other files.

```bash
# The * will be removed when searching so this says match anything 
# that starts with '' which is everything
# Since '--a[*]' will not match anything no sub-folders will be searched
qf * --a[*]
```

#### `--ra`
This flag effects where the `--a` flag applies. When the `--ra` flag is provided the `--a` flag only applies to the cwd.

For example lets say your file tree looks like this:
```text
root
├── modules
├── src
└── target
    └── release
        └── qf.exe
```

If I run `qf qf.exe` alone I will find the file I am looking for. However I had to search every file in modules and src to do so.

This is where you'd want to add the `--a` flag to optimize your search so try it out.
```bash
# only include target
qf qf.exe --a[target]

> Searching for qf.exe...
> Searched 7 files. Found 0 files.
> Completed in 559.60µs
```

This of course doesn't work because the folder `release` is preventing us from seeing the `qf.exe` file because it does not match the `target` directory.

This is where the `--r` flag is useful because it allows you to only apply the `--a` flag rule at the level of the cwd (Current working directory).

So lets try again but with the `--r` flag.

```bash
# only include target
qf qf.exe --a[target] --ra

> Searching for qf.exe...
> Searched 7 files. Found 1 files.
> Completed in 559.60µs
```

Now the `--a` rule applied on our cwd but does not apply to any sub-directories.

#### `--c[arg]`
The `--c` flag allows you to specify the minimum amount of entries (files/folders) required to be in a directory for the program to search them concurrently. Used correctly this can save some amount of time and resources.

Usage:
```bash
# Will require 20 files/folders 
# in a directory before using concurrency
qf index.* --c[20]
```

The default and minimum value is 2 as you can't split 1 or 0 entries onto concurrent threads.

As far as what to set this to it is really up to the amount of files you are searching. You may have to do your own testing to really find out.

#### `--dc`
The `--dc` flag allows you to disable concurrency altogether. This can be useful when searching smaller directories as concurrency may not be beneficial or if you have the time but want to save the resources. 

Usage:
```bash
qf index.* --dc
```

We obviously recommend you leave this on as when you are searching larger directories using concurrency is normally about 2 times faster.