use clap::Parser;
use itertools::Itertools as _;
use std::{fmt, path::PathBuf};
use wasmtime::{
    Engine, ExternType, FuncType, GlobalType, MemoryType, Module, Mutability, TableType, ValType,
};

#[derive(Parser)]
struct Args {
    file: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let Args { file } = Args::parse();
    let source = std::fs::read(file)?;
    let engine = Engine::default();
    let module = Module::new(&engine, source)?;
    for export in module.exports() {
        println!("{}: {}", export.name(), CustomDisplay::new(export.ty()));
    }
    Ok(())
}

struct CustomDisplay<T> {
    inner: T,
}

impl<T> CustomDisplay<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl fmt::Display for CustomDisplay<ValType> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.inner {
            ValType::I32 => f.write_str("i32"),
            ValType::I64 => f.write_str("i64"),
            ValType::F32 => f.write_str("f32"),
            ValType::F64 => f.write_str("f64"),
            ValType::V128 => f.write_str("v128"),
            ValType::FuncRef => f.write_str("$func"),
            ValType::ExternRef => f.write_str("$extern"),
        }
    }
}

impl fmt::Display for CustomDisplay<Mutability> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.inner {
            Mutability::Const => f.write_str("const"),
            Mutability::Var => f.write_str("var"),
        }
    }
}

impl fmt::Display for CustomDisplay<FuncType> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "fn({}) -> ({})",
            self.inner.params().map(CustomDisplay::new).join(", "),
            self.inner.results().map(CustomDisplay::new).join(", ")
        ))
    }
}

impl fmt::Display for CustomDisplay<GlobalType> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "global {} {}",
            CustomDisplay::new(self.inner.mutability()),
            CustomDisplay::new(self.inner.content().clone())
        ))
    }
}

impl fmt::Display for CustomDisplay<TableType> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "table {} [{}..",
            CustomDisplay::new(self.inner.element()),
            self.inner.minimum()
        ))?;
        if let Some(max) = self.inner.maximum() {
            f.write_fmt(format_args!("{}", max))?
        }
        f.write_str("]")
    }
}

impl fmt::Display for CustomDisplay<MemoryType> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.inner.is_shared() {
            f.write_str("shared ")?
        }
        f.write_str("memory ")?;
        match self.inner.is_64() {
            true => f.write_str("64")?,
            false => f.write_str("32")?,
        }
        f.write_fmt(format_args!(" [{}..", self.inner.minimum()))?;
        if let Some(max) = self.inner.maximum() {
            f.write_fmt(format_args!("{}", max))?
        }
        f.write_str("]")
    }
}

impl fmt::Display for CustomDisplay<ExternType> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.inner.clone() {
            ExternType::Func(it) => f.write_fmt(format_args!("{}", CustomDisplay::new(it))),
            ExternType::Global(it) => f.write_fmt(format_args!("{}", CustomDisplay::new(it))),
            ExternType::Table(it) => f.write_fmt(format_args!("{}", CustomDisplay::new(it))),
            ExternType::Memory(it) => f.write_fmt(format_args!("{}", CustomDisplay::new(it))),
        }
    }
}
