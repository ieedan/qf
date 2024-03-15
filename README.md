# qf
qf or (Quick Find) is a CLI file search implementation that makes file searching super simple.

### Basic Use
``` bash
# find all files that match 'main.js'
qf index.js
```

### Wildcards
**qf** allows for the use of wild cards to improve your search.
```bash
qf *.js # Find all files that end with '.js'

qf index* # Find all files that start with 'index'

qf *index* # Find all files that contain 'index'
```

### Flags
- `--i[args]` Ignore directories.
- `--a[args]` Allow directories.
- `--r` Only apply allow to root directory.

#### `--i[args]`
This allows you to skip searching files in the provided subdirectories. That looks something like this. 

```bash
qf index.js --i[node_modules]

# For folder names including a space you must wrap them in '' or ""
qf index.js --i['your name']
```

#### `--a[args]`
This allows you to only search directories that match the provided directories.

```bash
qf index.js --a[src]
```

This can also be useful if you just want to list the files in your cwd (Current working directory) without searching all other files.

```bash
# The * will be removed when searching so this says match anything 
# that starts with '' which is everything
# Since '*' will not match anything no sub-folders will be searched
qf * --a[*]
```

#### `--r`
This flag effects where the `--a` flag applies. When the `--r` flag is provided the `--a` flag only applies to the current working directory.

For example lets say your file tree looks like this:
```bash
qf
|--modules
|--src
|--target
   |--release
      |--qf.exe
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
qf qf.exe --a[target] --r

> Searching for qf.exe...
> Searched 7 files. Found 1 files.
> Completed in 559.60µs
```

Now the `--a` rule applied on our cwd but does not apply to any sub-directories.