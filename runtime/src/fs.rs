use maybe_static::maybe_static;
use rusty_jsc::{JSClass, JSContext, JSObject, JSObjectGeneric, JSValue};

use crate::fs_write_stream::create_write_stream;

#[allow(unused)]

/// Rust side of fsPromise.constants and fs.constants
pub mod constants {
    pub const UV_FS_SYMLINK_DIR: u16 = 1;
    pub const UV_FS_SYMLINK_JUNCTION: u16 = 2;
    pub const O_RDONLY: u16 = 0;
    pub const O_WRONLY: u16 = 1;
    pub const O_RDWR: u16 = 2;
    pub const UV_DIRENT_UNKNOWN: u16 = 0;
    pub const UV_DIRENT_FILE: u16 = 1;
    pub const UV_DIRENT_DIR: u16 = 2;
    pub const UV_DIRENT_LINK: u16 = 3;
    pub const UV_DIRENT_FIFO: u16 = 4;
    pub const UV_DIRENT_SOCKET: u16 = 5;
    pub const UV_DIRENT_CHAR: u16 = 6;
    pub const UV_DIRENT_BLOCK: u16 = 7;
    pub const S_IFMT: u16 = 61440;
    pub const S_IFREG: u16 = 32768;
    pub const S_IFDIR: u16 = 16384;
    pub const S_IFCHR: u16 = 8192;
    pub const S_IFBLK: u16 = 24576;
    pub const S_IFIFO: u16 = 4096;
    pub const S_IFLNK: u16 = 40960;
    pub const S_IFSOCK: u16 = 49152;
    pub const O_CREAT: u16 = 64;
    pub const O_EXCL: u16 = 128;
    pub const UV_FS_O_FILEMAP: u16 = 0;
    pub const O_NOCTTY: u16 = 256;
    pub const O_TRUNC: u16 = 512;
    pub const O_APPEND: u16 = 1024;
    pub const O_DIRECTORY: u32 = 65536;
    pub const O_NOATIME: u32 = 262144;
    pub const O_NOFOLLOW: u32 = 131072;
    pub const O_SYNC: u32 = 1052672;
    pub const O_DSYNC: u16 = 4096;
    pub const O_DIRECT: u16 = 16384;
    pub const O_NONBLOCK: u16 = 2048;
    pub const S_IRWXU: u16 = 448;
    pub const S_IRUSR: u16 = 256;
    pub const S_IWUSR: u16 = 128;
    pub const S_IXUSR: u16 = 64;
    pub const S_IRWXG: u16 = 56;
    pub const S_IRGRP: u16 = 32;
    pub const S_IWGRP: u16 = 16;
    pub const S_IXGRP: u16 = 8;
    pub const S_IRWXO: u16 = 7;
    pub const S_IROTH: u16 = 4;
    pub const S_IWOTH: u16 = 2;
    pub const S_IXOTH: u16 = 1;
    pub const F_OK: u8 = 0;
    pub const R_OK: u8 = 4;
    pub const W_OK: u8 = 2;
    pub const X_OK: u8 = 1;
    pub const UV_FS_COPYFILE_EXCL: u16 = 1;
    pub const COPYFILE_EXCL: u16 = 1;
    pub const UV_FS_COPYFILE_FICLONE: u16 = 2;
    pub const COPYFILE_FICLONE: u16 = 2;
    pub const UV_FS_COPYFILE_FICLONE_FORCE: u16 = 4;
    pub const COPYFILE_FICLONE_FORCE: u16 = 4;
}

pub fn constants_object(context: &JSContext) -> JSObject {
    /// Macro to set constant property
    macro_rules! cst_prop {
        ($obj: ident, $ctx: ident, $( $name: ident ),*) => {
            $(
            $obj.set_property(
                $ctx,
                stringify!($name),
                JSValue::number($ctx, constants::$name as f64),
            )
            .unwrap();
            )*
        };
    }
    let mut obj = JSObject::<JSObjectGeneric>::new(context);
    cst_prop!(
        obj,
        context,
        UV_FS_SYMLINK_DIR,
        UV_FS_SYMLINK_JUNCTION,
        O_RDONLY,
        O_WRONLY,
        O_RDWR,
        UV_DIRENT_UNKNOWN,
        UV_DIRENT_FILE,
        UV_DIRENT_DIR,
        UV_DIRENT_LINK,
        UV_DIRENT_FIFO,
        UV_DIRENT_SOCKET,
        UV_DIRENT_CHAR,
        UV_DIRENT_BLOCK,
        S_IFMT,
        S_IFREG,
        S_IFDIR,
        S_IFCHR,
        S_IFBLK,
        S_IFIFO,
        S_IFLNK,
        S_IFSOCK,
        O_CREAT,
        O_EXCL,
        UV_FS_O_FILEMAP,
        O_NOCTTY,
        O_TRUNC,
        O_APPEND,
        O_DIRECTORY,
        O_NOATIME,
        O_NOFOLLOW,
        O_SYNC,
        O_DSYNC,
        O_DIRECT,
        O_NONBLOCK,
        S_IRWXU,
        S_IRUSR,
        S_IWUSR,
        S_IXUSR,
        S_IRWXG,
        S_IRGRP,
        S_IWGRP,
        S_IXGRP,
        S_IRWXO,
        S_IROTH,
        S_IWOTH,
        S_IXOTH,
        F_OK,
        R_OK,
        W_OK,
        X_OK,
        UV_FS_COPYFILE_EXCL,
        COPYFILE_EXCL,
        UV_FS_COPYFILE_FICLONE,
        COPYFILE_FICLONE,
        UV_FS_COPYFILE_FICLONE_FORCE,
        COPYFILE_FICLONE_FORCE
    );
    obj
}

pub fn fs(context: &JSContext) -> JSObject {
    let fs_class = maybe_static!(JSClass, || JSClass::create("FileSystem", None, None));
    let mut fp = fs_class.make_object(context);

    fp.set_property(
        context,
        "createWriteStream",
        JSValue::callback(context, Some(create_write_stream)),
    )
    .unwrap();
    fp.into()
}
