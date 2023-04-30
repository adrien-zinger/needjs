use rusty_jsc::{JSContext, JSObject, JSObjectGeneric, JSValue};

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
    let mut obj = JSObject::<JSObjectGeneric>::make(context);
    obj.set_property(
        context,
        "UV_FS_SYMLINK_DIR",
        JSValue::number(context, constants::UV_FS_SYMLINK_DIR as f64),
    );

    obj.set_property(
        context,
        "UV_FS_SYMLINK_JUNCTION",
        JSValue::number(context, constants::UV_FS_SYMLINK_JUNCTION as f64),
    );
    obj.set_property(
        context,
        "O_RDONLY",
        JSValue::number(context, constants::O_RDONLY as f64),
    );
    obj.set_property(
        context,
        "O_WRONLY",
        JSValue::number(context, constants::O_WRONLY as f64),
    );
    obj.set_property(
        context,
        "O_RDWR",
        JSValue::number(context, constants::O_RDWR as f64),
    );
    obj.set_property(
        context,
        "UV_DIRENT_UNKNOWN",
        JSValue::number(context, constants::UV_DIRENT_UNKNOWN as f64),
    );
    obj.set_property(
        context,
        "UV_DIRENT_FILE",
        JSValue::number(context, constants::UV_DIRENT_FILE as f64),
    );
    obj.set_property(
        context,
        "UV_DIRENT_DIR",
        JSValue::number(context, constants::UV_DIRENT_DIR as f64),
    );
    obj.set_property(
        context,
        "UV_DIRENT_LINK",
        JSValue::number(context, constants::UV_DIRENT_LINK as f64),
    );
    obj.set_property(
        context,
        "UV_DIRENT_FIFO",
        JSValue::number(context, constants::UV_DIRENT_FIFO as f64),
    );
    obj.set_property(
        context,
        "UV_DIRENT_SOCKET",
        JSValue::number(context, constants::UV_DIRENT_SOCKET as f64),
    );
    obj.set_property(
        context,
        "UV_DIRENT_CHAR",
        JSValue::number(context, constants::UV_DIRENT_CHAR as f64),
    );
    obj.set_property(
        context,
        "UV_DIRENT_BLOCK",
        JSValue::number(context, constants::UV_DIRENT_BLOCK as f64),
    );
    obj.set_property(
        context,
        "S_IFMT",
        JSValue::number(context, constants::S_IFMT as f64),
    );
    obj.set_property(
        context,
        "S_IFREG",
        JSValue::number(context, constants::S_IFREG as f64),
    );
    obj.set_property(
        context,
        "S_IFDIR",
        JSValue::number(context, constants::S_IFDIR as f64),
    );
    obj.set_property(
        context,
        "S_IFCHR",
        JSValue::number(context, constants::S_IFCHR as f64),
    );
    obj.set_property(
        context,
        "S_IFBLK",
        JSValue::number(context, constants::S_IFBLK as f64),
    );
    obj.set_property(
        context,
        "S_IFIFO",
        JSValue::number(context, constants::S_IFIFO as f64),
    );
    obj.set_property(
        context,
        "S_IFLNK",
        JSValue::number(context, constants::S_IFLNK as f64),
    );
    obj.set_property(
        context,
        "S_IFSOCK",
        JSValue::number(context, constants::S_IFSOCK as f64),
    );
    obj.set_property(
        context,
        "O_CREAT",
        JSValue::number(context, constants::O_CREAT as f64),
    );
    obj.set_property(
        context,
        "O_EXCL",
        JSValue::number(context, constants::O_EXCL as f64),
    );
    obj.set_property(
        context,
        "UV_FS_O_FILEMAP",
        JSValue::number(context, constants::UV_FS_O_FILEMAP as f64),
    );
    obj.set_property(
        context,
        "O_NOCTTY",
        JSValue::number(context, constants::O_NOCTTY as f64),
    );
    obj.set_property(
        context,
        "O_TRUNC",
        JSValue::number(context, constants::O_TRUNC as f64),
    );
    obj.set_property(
        context,
        "O_APPEND",
        JSValue::number(context, constants::O_APPEND as f64),
    );
    obj.set_property(
        context,
        "O_DIRECTORY",
        JSValue::number(context, constants::O_DIRECTORY as f64),
    );
    obj.set_property(
        context,
        "O_NOATIME",
        JSValue::number(context, constants::O_NOATIME as f64),
    );
    obj.set_property(
        context,
        "O_NOFOLLOW",
        JSValue::number(context, constants::O_NOFOLLOW as f64),
    );
    obj.set_property(
        context,
        "O_SYNC",
        JSValue::number(context, constants::O_SYNC as f64),
    );
    obj.set_property(
        context,
        "O_DSYNC",
        JSValue::number(context, constants::O_DSYNC as f64),
    );
    obj.set_property(
        context,
        "O_DIRECT",
        JSValue::number(context, constants::O_DIRECT as f64),
    );
    obj.set_property(
        context,
        "O_NONBLOCK",
        JSValue::number(context, constants::O_NONBLOCK as f64),
    );
    obj.set_property(
        context,
        "S_IRWXU",
        JSValue::number(context, constants::S_IRWXU as f64),
    );
    obj.set_property(
        context,
        "S_IRUSR",
        JSValue::number(context, constants::S_IRUSR as f64),
    );
    obj.set_property(
        context,
        "S_IWUSR",
        JSValue::number(context, constants::S_IWUSR as f64),
    );
    obj.set_property(
        context,
        "S_IXUSR",
        JSValue::number(context, constants::S_IXUSR as f64),
    );
    obj.set_property(
        context,
        "S_IRWXG",
        JSValue::number(context, constants::S_IRWXG as f64),
    );
    obj.set_property(
        context,
        "S_IRGRP",
        JSValue::number(context, constants::S_IRGRP as f64),
    );
    obj.set_property(
        context,
        "S_IWGRP",
        JSValue::number(context, constants::S_IWGRP as f64),
    );
    obj.set_property(
        context,
        "S_IXGRP",
        JSValue::number(context, constants::S_IXGRP as f64),
    );
    obj.set_property(
        context,
        "S_IRWXO",
        JSValue::number(context, constants::S_IRWXO as f64),
    );
    obj.set_property(
        context,
        "S_IROTH",
        JSValue::number(context, constants::S_IROTH as f64),
    );
    obj.set_property(
        context,
        "S_IWOTH",
        JSValue::number(context, constants::S_IWOTH as f64),
    );
    obj.set_property(
        context,
        "S_IXOTH",
        JSValue::number(context, constants::S_IXOTH as f64),
    );
    obj.set_property(
        context,
        "F_OK",
        JSValue::number(context, constants::F_OK as f64),
    );
    obj.set_property(
        context,
        "R_OK",
        JSValue::number(context, constants::R_OK as f64),
    );
    obj.set_property(
        context,
        "W_OK",
        JSValue::number(context, constants::W_OK as f64),
    );
    obj.set_property(
        context,
        "X_OK",
        JSValue::number(context, constants::X_OK as f64),
    );
    obj.set_property(
        context,
        "UV_FS_COPYFILE_EXCL",
        JSValue::number(context, constants::UV_FS_COPYFILE_EXCL as f64),
    );
    obj.set_property(
        context,
        "COPYFILE_EXCL",
        JSValue::number(context, constants::COPYFILE_EXCL as f64),
    );
    obj.set_property(
        context,
        "UV_FS_COPYFILE_FICLONE",
        JSValue::number(context, constants::UV_FS_COPYFILE_FICLONE as f64),
    );
    obj.set_property(
        context,
        "COPYFILE_FICLONE",
        JSValue::number(context, constants::COPYFILE_FICLONE as f64),
    );
    obj.set_property(
        context,
        "UV_FS_COPYFILE_FICLONE_FORCE",
        JSValue::number(context, constants::UV_FS_COPYFILE_FICLONE_FORCE as f64),
    );
    obj.set_property(
        context,
        "COPYFILE_FICLONE_FORCE",
        JSValue::number(context, constants::COPYFILE_FICLONE_FORCE as f64),
    );
    obj
}
