use std::{time::Duration, sync::{Arc, Mutex}, mem, pin::Pin, future::Future, collections::HashSet};
use async_std::task::{self, block_on};
use crate::{controller::{globalcontext::{GlobalBuildContext, GlobalContext}, operation::{Return, Operation}, interpreter::{Interpreter, InterpreterBuilder}, objectcode::{Metadata, CompiledBlock, ObjectFile}, context::RunContext, value::Value, program}, libcore::libcore::{LibcoreTemplate, build_libcore, prepare_libcore, LibcoreBuilder}};

#[derive(Clone)]
struct LibcoreTest {
    printed: Arc<Mutex<Vec<String>>>,
    asyncs: Arc<Mutex<u64>>
}

async fn call_up_async(asyncs: Arc<Mutex<u64>>) -> Result<(),String> {
    eprintln!("waiting");
    task::sleep(Duration::from_millis(1000)).await;
    eprintln!("/waiting");
    *asyncs.lock().unwrap() += 1;
    Ok(())
}

impl LibcoreTemplate for LibcoreTest {
    fn print(&self, s: &str) {
        self.printed.lock().unwrap().push(s.to_string());
    }

    fn call_up(&self) -> Pin<Box<dyn Future<Output=Result<(),String>>>> {
        let asyncs = self.asyncs.clone();
        Box::pin(call_up_async(asyncs.clone()))
    }
}

fn printed(context: &LibcoreTest) -> Vec<String> {
    mem::replace(context.printed.lock().unwrap().as_mut(),vec![])
}

fn prepare_interpreter() -> (Interpreter,LibcoreBuilder) {
    let mut builder = InterpreterBuilder::new();
    let libcore = build_libcore(&mut builder).expect("build failed");
    let interp = Interpreter::new(builder);
    (interp,libcore)
}

fn run_interpreter(interp: Interpreter, libcore: LibcoreBuilder) -> LibcoreTest {
    let mut context = RunContext::new();
    let libcore_context = LibcoreTest {
        printed: Arc::new(Mutex::new(vec![])),
        asyncs: Arc::new(Mutex::new(0))
    };
    prepare_libcore(&mut context,&libcore,libcore_context.clone());
    block_on(interp.run(&Metadata::new("group","program",1),"main",context)).expect("run");
    libcore_context
}

fn count_opcodes(file: &ObjectFile) {
    let mut seen = HashSet::new();
    for program in &file.code {
        for (_,block) in &program.code {
            for (opcode,_) in &block.program {
                seen.insert(opcode);
            }
        }
    }
    let mut seen = seen.drain().collect::<Vec<_>>();
    seen.sort();
    let seen = seen.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    eprintln!("seen opcodes {}",seen.join(", "));
}

#[test]
fn test_load() {
    let (mut interp,libcore) = prepare_interpreter();
    interp.load(include_bytes!("hello.eardo")).expect("adding");
    let output = run_interpreter(interp,libcore);
    let file = ObjectFile::decode(include_bytes!("hello.eardo").to_vec()).expect("decoding");
    count_opcodes(&file);
    let out = printed(&output);
    assert_eq!(vec![
        "hello, world!",
        "[1, 2, 3]",
        "[\"hi\",...]",
        "[1, 2, 3]",
        "[true, false]",
        "[\"hello\", \"world\"]",
        "[1, 2, 3]",
        "[true, false]",
        "[\"hello\", \"world\"]",
        "[6,...]",
        "[true,...]",
        "[\"boo\",...]"
    ],out);
}
