// Copyright 2020 Trezoa Maintainers <maintainers@trezoa.com>
//
// Licensed under the Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> or
// the MIT license <http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::{fs::File, io::Read, sync::Arc};
use trezoa_rbpf::{
    elf::Executable,
    program::{BuiltinFunction, BuiltinProgram, FunctionRegistry},
    syscalls,
    vm::{Config, TestContextObject},
};

fn loader() -> Arc<BuiltinProgram<TestContextObject>> {
    let mut function_registry = FunctionRegistry::<BuiltinFunction<TestContextObject>>::default();
    function_registry
        .register_function_hashed(*b"log", syscalls::SyscallString::vm)
        .unwrap();
    Arc::new(BuiltinProgram::new_loader(
        Config::default(),
        function_registry,
    ))
}

fn bench_load_sbpfv0(c: &mut Criterion) {
    let mut file = File::open("tests/elfs/syscall_reloc_64_32_sbpfv0.so").unwrap();
    let mut elf = Vec::new();
    file.read_to_end(&mut elf).unwrap();
    let loader = loader();
    c.bench_function("load_sbpfv0", |b| {
        b.iter(|| {
            Executable::<TestContextObject>::from_elf(black_box(&elf), loader.clone()).unwrap()
        })
    });
}

criterion_group!(benches, bench_load_sbpfv0);
criterion_main!(benches);
