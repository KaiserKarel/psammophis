use std::collections::HashSet;
use std::path::PathBuf;
use thiserror::Error;


/// Privilege level over action set T.
#[derive(Clone)]
pub enum Privilege<T> {
    /// Vm may perform action for capabilities in T.
    Limited(T),

    /// Vm is completely unrestricted; i.e may access all files.
    Total,

    /// Vm may not perform any action of this type.
    None,
}

impl<T> Default for Privilege<T> {
    fn default() -> Self {
        Self::None
    }
}


/// Capability describes the set of capabilities a VM has.
#[derive(Default, Clone)]
pub struct Capability {
    /// Files/directories to which the VM has certain privileges. See
    /// fs::Capability for the specifics. If a directory is provided,
    /// all files and directories inside it may be accessed.
    pub fs: Privilege<HashSet<fs::Capability>>,

    /// URLs which the python code may make http requests to.
    pub http: Privilege<HashSet<http::Capability>>,
}

impl Capability {
    pub fn add<A: Add>(&mut self, item: A) -> Result<(), &str> {
        item.add(self)
    }
}

pub trait Add {
    fn add(self, cap: &mut Capability) -> Result<(), &str>;
}

pub struct Http;

impl Http {
    pub fn url(input: &str) -> Result<http::Capability, url::ParseError> {
        Ok(http::Capability{
            url: url::Url::parse(input)?
        })
    }
}

pub struct Fs;

impl Fs {
    pub fn dir<P: Into<PathBuf>>(op: fs::Operation, dir: P) -> Result<fs::Capability, FsDirError> {
        let path = dir.into();
        if !path.is_dir() {
            return Err(FsDirError::IsNoDir)
        }
        Ok(fs::Capability{path, op})
    }

    pub fn file<P: Into<PathBuf>>(op: fs::Operation, file: P) -> Result<fs::Capability, FsDirError> {
        let path = file.into();
        if !path.is_file() {
            return Err(FsDirError::IsNoFile)
        }
        Ok(fs::Capability{path, op})
    }
}

#[derive(Error, Debug)]
pub enum FsDirError {
    #[error("not a directory")]
    IsNoDir,
    #[error("not a file")]
    IsNoFile,
}

pub mod http {
    #[derive(Clone)]
    pub struct Capability {
        pub url: url::Url,
    }
}

pub mod fs {
    use std::path::PathBuf;
    use thiserror::Error;

    use super::Add;

    #[derive(Clone, PartialEq, Eq, Hash)]
    pub enum Operation {
        Read,
        Write,
        ReadWrite,
        Append,
        ReadAppend,
    }

    #[derive(Error, Debug)]
    pub struct ParseOperationError;

    impl std::fmt::Display for ParseOperationError {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
            write!(f, "cannot parse file operation")
        }
    }


    use std::str::FromStr;
    use std::fmt::Formatter;
    use std::convert::TryInto;

    /// Uses the python conventions for file access
    /// 'r'     -> Read
    /// 'r+'    -> Write
    impl FromStr for Operation {
        type Err = ParseOperationError;
        fn from_str(token: &str) -> Result<Self, Self::Err> {
            match token {
                "r" => Ok(Operation::Read),
                "rb" => Ok(Operation::Read),
                "r+" => Ok(Operation::ReadWrite),
                "rb+" => Ok(Operation::ReadWrite),
                "w" => Ok(Operation::Write),
                "wb" => Ok(Operation::Write),
                "wb+" => Ok(Operation::ReadWrite),
                "a" => Ok(Operation::Append),
                "ab" => Ok(Operation::Append),
                "a+" => Ok(Operation::ReadAppend),
                "ab+" => Ok(Operation::ReadAppend),
                _ => Err(ParseOperationError)
            }
        }
    }

    use std::convert::TryFrom;

    impl TryFrom<&str> for Operation {
        type Error = ParseOperationError;

        fn try_from(value: &str) -> Result<Self, Self::Error> {
            Operation::from_str(value)
        }
    }

    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct Capability {
        pub path: PathBuf,
        pub op: Operation
    }

    use super::Privilege;

    impl Add for Capability {
        fn add(self, cap: &mut super::Capability) -> Result<(), &str> {
            match &mut cap.fs {
                Privilege::None => Err("incompatible capabilities"),
                Privilege::Total => Ok(()),
                Privilege::Limited(ref mut set) => {
                    set.insert(self);
                    Ok(())
                }
                }
            }
        }

    pub enum AccessError {

    }

    impl Capability {
        pub fn has_access<P: Into<PathBuf>, O: TryInto<Operation, Error=ParseOperationError>>(&self, path: P , op: O) -> Result<bool, ParseOperationError> {
            let path = path.into();
            let op = op.try_into()?;
            todo!()
        }
    }
}