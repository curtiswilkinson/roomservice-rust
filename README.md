# roomservice-rust
The canonical roomservice repository, replacing roomservice JS

Roomservice is a small, friendly build tool that uses file system timestamps to
determine if a directory needs building, and build it according to the config.

![Roomservice Example](https://raw.githubusercontent.com/curtiswilkinson/roomservice/master/images/example.gif)

## Use case

This project was born out of working in an application containing many,
frequently changing microservices. When doing things such as pulling from
version control, it was painful to have to rebuild the entire world (or try and
figure out what needs to be built).

Roomservice solves this problem by keeping a cache of the last time it built a
room, and doing very quick diffing to determine what actually needs to be built.

## Getting started

Please note the second command maybe require sudo in order to move the binary into /usr/local/bin

### Linux

```sh
curl -L https://github.com/curtiswilkinson/roomservice-rust/releases/download/v4.0.1/x86_64-unknown-linux-musl.tar.gz | tar xz
 
cp target/x86_64-unknown-linux-musl/release/roomservice /usr/local/bin && rm -rf target roomservice.tar.gz
```

### OSX (x86)

```sh
curl -L https://github.com/curtiswilkinson/roomservice-rust/releases/download/v4.0.1/x86_64-apple-darwin.tar.gz | tar xz
 
cp target/x86_64-apple-darwin/roomservice /usr/local/bin && rm -rf target roomservice.tar.gz

```

### OSX (ARM)

```sh
curl -L https://github.com/curtiswilkinson/roomservice-rust/releases/download/v4.0.1/aarch64-apple-darwin.tar.gz | tar xz
 
cp target/aarch64-apple-darwin/roomservice /usr/local/bin && rm -rf target roomservice.tar.gz

```
## Config

Roomservice supports YAML, TOML & JSON formats, but the current encouraged
default is YAML. The structure for other file formats is identical, to see some
examples look in the `./mock` folder.

Here is what roomservice config looks like:

```yaml
rooms:
  room_name:
    path: ./path/to/watch/and/run/commands/in
    before: 'runs before everything'
    runParallel: 'runs in parallel with other rooms'
    runSynchronous: 'runs synchronously with other rooms'
    after: 'run after the run commands'
    finally: 'ALWAYS runs last, regardless of directory changes'
```

_Note:_ All commands are run in the `path` provided, adjust any relative paths
accordingly

So for example, a project with two docker services might look like this to avoid
the known speed issues with parallel container builds:

```yaml
rooms:
  api:
    path: ./api
    before: yarn && yarn build
    runSynchronously: docker-compose stop api && docker-compose build api
    finally: docker-compose up -d api

  client:
    path: ./client
    before: yarn && yarn build
    runSynchronously: docker-compose stop client && docker-compose build client
    finally: docker-compose up -d client
```

This would build as follows (assuming no roomservice cache saves you!):

1. Both `before` commands will start running at the same time
2. Roomservice will wait for both `before` commands to complete, then move on
3. Because the `runSynchronous` command is synchronous, it will do one
   `docker-compose build` first, then do the second
4. Lastly, the `finally` command will fire, calling `docker-compose up` on both
   containers

In the event that no files in either path had changed:

1. Only the `finally` hooks would file, simply calling `docker-compose up` on
   both containers

## CLI

* `--help` will list all available commands
* `--project` or `-p` allows you to provide the path to the roomservice project
  or config file
* `--init` will create a `roomservice.config.toml` file
* `--no-cache` will skip all caching steps and build all rooms
* `--cache-all` will not build anything, but flag all rooms as updated (good for
  the initial setup)
* `--ignore` will take a list of room names, and ignore them during the build
* `--only` will take a list of room names, and only build those rooms
* `--no-finally` will skip the finally hook when building


