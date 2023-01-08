use std::{time::Duration, sync::{Arc, Mutex}, mem, pin::Pin, future::Future, collections::HashSet};
use async_std::task::{self, block_on};
use crate::{controller::{interpreter::{Interpreter, InterpreterBuilder}, objectcode::{ProgramName, ObjectFile}, context::RunContext}, libcore::libcore::{LibcoreTemplate, build_libcore, prepare_libcore, LibcoreBuilder}};

#[derive(Clone)]
struct LibcoreTest {
    printed: Arc<Mutex<Vec<String>>>,
    asyncs: Arc<Mutex<u64>>
}

async fn call_up_async(asyncs: Arc<Mutex<u64>>) -> Result<(),String> {
    eprintln!("waiting");
    task::sleep(Duration::from_millis(100)).await;
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

fn run_interpreter(interp: &Interpreter, libcore: &LibcoreBuilder, part: &str) -> Vec<String> {
    let mut context = RunContext::new();
    let libcore_context = LibcoreTest {
        printed: Arc::new(Mutex::new(vec![])),
        asyncs: Arc::new(Mutex::new(0))
    };
    prepare_libcore(&mut context,&libcore,libcore_context.clone());
    let program = interp.get(&ProgramName::new("group","program",1),part).expect("load failed");
    match block_on(program.run(context)) {
        Ok(_) => printed(&libcore_context),
        Err(e) => vec![e],
    }
}

fn count_opcodes(bytes: &[u8]) {
    let file = ObjectFile::decode(bytes.to_vec()).expect("decoding");
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
    interp.load(include_bytes!("smoke.eardo")).expect("adding");
    let out = run_interpreter(&interp,&libcore,"main");
    count_opcodes(include_bytes!("smoke.eardo"));
    assert_eq!(vec![
        "\"hello, world!\"",
        "[1,2,3]",
        "[\"hi\",...]",
        "[1,2,3]",
        "[true,false]",
        "[\"hello\",\"world\"]",
        "[1,2,3]",
        "[true,false]",
        "[\"hello\",\"world\"]",
        "[6,...]",
        "[true,...]",
        "[\"boo\",...]",
        "2", "-1", "3", "3", "2", "2",
        "[2,2,5]", "[1,1,3]", "[2,2,3]", "[1,2,2]", "[2,2,3]", "[1,2,2]",
        "[2,2,5]", "[1,1,3]", "[2,2,3]", "[1,2,2]", "[2,2,3]", "[1,2,2]",
        "[2,2,5]", "[1,1,3]", "[2,2,3]", "[1,2,2]", "[2,2,3]", "[1,2,2]",
        "5", "[4,5]", "[4,6]", "[6.2,7.2]",
        "-1", "[-2,-1]", "[-2,-2]", "[-4.2,-3.2]",
        "6", "[3,6]", "[3,8]", "[5.2,10.4]", 
        "9.5", "[0.5,1]", "[0.5,0.5]", "[4,8]",
        "1", "[1,0]", "[1,2]", "[0,1]",

        "true", "false", "false", 
        "true", "true", "false", 

        "[false,false,true,true]", "[false,false,false,true]", "[false,false,false,false]", 
        "[false,true,true,true]", "[false,false,true,true]", "[false,false,false,true]", 

        "[false,false,false,true]", "[false,false,true,true]",

        "true", "false", "[true,false]", "[true,false,false,true]",
        "true", "false", "[true,false]", "[true,false,false,true]", 
        "true", "false", "[true,false]", "[true,false,false,true]",
        "false", "true", "[false,true]", "[false,false,false,true]",
        "false", "true", "[false,true]", "[false,true,true,true]",

        "-6", "[2,1,-1,-2]",

        "true", "[false,true,true,false]",

        "[true,true,true,true,true]", "[]", "[3,3,3,3,3,3,3,3,3,3]",
        "[6,7,8,9,0]", "[1,2,3,4,5]", 
        
        "[6,7,3,9,5]", "[1,2,3,4,5]", "[6,7,8,9,0]",
        "[-1,-1,3,-1,5]", "[1,2,3,4,5]", "[-1,...]",

        "[6,7,3,8,5]", "[1,2,3,4,5]", "[6,7,8,9,0]",
        "[-1,-1,3,-1,5]", "[1,2,3,4,5]", "[-1,...]",

        "[1,2,6,4,7]", "[1,2,-1,4,-1]",

        "[8,0,3,4,5]", "[-1,-1,3,4,5]",

        "3", "[3,4,5]",

        "[0,1,1,1,1,1,1,1,2,2,2,2,3,3]",
        "[0,0,1,2,3,4,5,6,0,1,2,3,0,1]",

        "\"\"", "\"hello\"", "\"hello,world\"",

        "\"hello, world\"", "[\"hello, world\",\"goodbye, world\"]", "[\"hello, world\",...]",
        "[\"hello, mercury\",\"hello, venus\",\"hello, earth\",\"hello, mars\"]",
        "[\"1\",\"2\",\"3\",\"4\",\"5\"]",
        "\"first: 1st; second: 2nd\"",

        "[\"fred\",\"wilma\",\"barney\",\"pebbles\"]",
        "[\"47\",\"39\",\"52\",\"2\"]",

        "[\"salt/pepper\",\"raw/cooked\",\"blake/avon\"]",

        "false", "false", "true", "true", "true", "false",

        "[1,3]",

        "true", "[false,true]", "true", "[false,true]", "true", "[false,true]",
        "\"true\"", "[\"false\",\"true\"]", "\"3\"", "[\"0\",\"1\"]", "\"8\"", "[\"\",\"8\"]",

        "1", "[0,1]", "3", "[0,1]", "8", "[0,8]",

        "3", "2", "3", "[0,1,2]", "5",
        "[false,true,true]", "true",

        "1", "6",

        "\"GcTan\"", "[\"GcTan\",\"TnNnT\"]",

        "[2,3,5]"

    ],out);
}

fn to_string_vec(input: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(input).expect("bad json")
}

fn to_number_vec(input: &str) -> Vec<usize> {
    serde_json::from_str::<Vec<usize>>(input).expect("bad json")
}

fn to_string_vecvec(input: &[String], count: &[usize]) -> Vec<Vec<String>> {
    let mut out = vec![];
    let mut pos = 0;
    for num in count {
        let mut here = vec![];
        for _ in 0..*num {
            here.push(input[pos].clone());
            pos += 1;
        }
        out.push(here);
    }
    out
}

fn run_check(interp: &Interpreter, libcore: &LibcoreBuilder, program: &str, compare: &[String]) {
    eprintln!("running {}",program);
    let output = &run_interpreter(&interp,&libcore,program);
    assert_eq!(output,compare);
}

#[test]
fn test_check() {
    let (mut interp,libcore) = prepare_interpreter();
    interp.load(include_bytes!("check.eardo")).expect("adding");
    count_opcodes(include_bytes!("check.eardo"));
    let out = run_interpreter(&interp,&libcore,"main");
    let program = to_string_vec(&out[0]);
    let count = to_number_vec(&out[1]);
    let compare = to_string_vec(&out[2]);
    let test_vector = to_string_vecvec(&compare,&count);
    for (program,compare) in program.iter().zip(test_vector.iter()) {
        run_check(&interp,&libcore,program,compare);
    }
}
