#![forbid(unsafe_code)]

use std::io::Read;
use std::{fs, io, path::Path};

////////////////////////////////////////////////////////////////////////////////

type Callback<'a> = dyn FnMut(&mut Handle) + 'a;

#[derive(Default)]
pub struct Walker<'a> {
    callbacks: Vec<Box<Callback<'a>>>,
}

impl<'a> Walker<'a> {
    pub fn new() -> Self {
        Self {
            callbacks: Vec::new(),
        }
    }

    pub fn add_callback<F>(&mut self, callback: F)
    where
        F: FnMut(&mut Handle) + 'a,
    {
        self.callbacks.push(Box::new(callback));
    }

    pub fn walk<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        if self.callbacks.is_empty() {
            return Ok(());
        }
        Self::recursive_walk(path.as_ref(), self.callbacks.as_mut_slice())?;
        Ok(())
    }

    fn recursive_walk(dir: &Path, callbacks: &mut [Box<Callback>]) -> io::Result<()> {
        if !dir.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "directory not found",
            ));
        }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let mut handle: Handle = {
                if path.is_file() {
                    Handle::File(FileHandle {
                        path: &path,
                        flag: false,
                    })
                } else if path.is_dir() {
                    Handle::Dir(DirHandle {
                        path: &path,
                        flag: false,
                    })
                } else {
                    continue;
                }
            };
            let mut anchor = 0;
            for i in 0..callbacks.len() {
                (callbacks[i])(&mut handle);
                if was_checked(&mut handle) {
                    if anchor < i {
                        callbacks.swap(anchor, i);
                    }
                    anchor += 1;
                }
            }
            if anchor == 0 {
                continue;
            }
            match handle {
                Handle::Dir(dir_handle) => {
                    Self::recursive_walk(dir_handle.path(), &mut callbacks[0..anchor])?;
                }
                Handle::File(file_handle) => {
                    let mut file = fs::File::open(file_handle.path())?;
                    let mut buf: Vec<u8> = Vec::new();
                    file.read_to_end(&mut buf)?;
                    let mut content_handle = Handle::Content {
                        file_path: file_handle.path(),
                        content: &buf,
                    };
                    for callback in callbacks.iter_mut().take(anchor) {
                        (callback)(&mut content_handle);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

pub enum Handle<'a> {
    Dir(DirHandle<'a>),
    File(FileHandle<'a>),
    Content {
        file_path: &'a Path,
        content: &'a [u8],
    },
}

pub fn was_checked(handle: &mut Handle) -> bool {
    match handle {
        Handle::Dir(dir) => {
            let ans = dir.flag;
            dir.flag = false;
            ans
        }
        Handle::File(file) => {
            let ans = file.flag;
            file.flag = false;
            ans
        }
        _ => false,
    }
}

pub struct DirHandle<'a> {
    path: &'a Path,
    flag: bool,
}

impl<'a> DirHandle<'a> {
    pub fn descend(&mut self) {
        self.flag = true;
    }

    pub fn path(&self) -> &Path {
        self.path
    }
}

pub struct FileHandle<'a> {
    path: &'a Path,
    flag: bool,
}

impl<'a> FileHandle<'a> {
    pub fn read(&mut self) {
        self.flag = true;
    }

    pub fn path(&self) -> &Path {
        self.path
    }
}
