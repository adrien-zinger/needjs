# File System development

We will focus on the asynchronous promise library. It will lead to the development of the internal event loop of the runtime itself and the development of the promise feature in rusty_jsc.

- [] fsPromises.access(path[, mode]) -- development
- [] fsPromises.appendFile(path, data[, options]) -- require Buffer to be full
- [] fsPromises.chmod(path, mode)
- [] fsPromises.chown(path, uid, gid)
- [] fsPromises.copyFile(src, dest[, mode])
- [] fsPromises.cp(src, dest[, options])
- [] fsPromises.lchmod(path, mode)
- [] fsPromises.lchown(path, uid, gid)
- [] fsPromises.lutimes(path, atime, mtime)
- [] fsPromises.link(existingPath, newPath)
- [] fsPromises.lstat(path[, options])
- [] fsPromises.mkdir(path[, options])
- [] fsPromises.mkdtemp(prefix[, options])
- [] fsPromises.open(path, flags[, mode]) -- development
- [] fsPromises.opendir(path[, options])
- [] fsPromises.readdir(path[, options])
- [] fsPromises.readFile(path[, options])
- [] fsPromises.readlink(path[, options])
- [] fsPromises.realpath(path[, options])
- [] fsPromises.rename(oldPath, newPath)
- [] fsPromises.rmdir(path[, options])
- [] fsPromises.rm(path[, options])
- [] fsPromises.stat(path[, options])
- [] fsPromises.statfs(path[, options])
- [] fsPromises.symlink(target, path[, type])
- [] fsPromises.truncate(path[, len])
- [] fsPromises.unlink(path)
- [] fsPromises.utimes(path, atime, mtime)
- [] fsPromises.watch(filename[, options])
- [] fsPromises.writeFile(file, data[, options])
- [x] fsPromises.constants -- working currently