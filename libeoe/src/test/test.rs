use std::{sync::{Mutex, Arc}, pin::Pin, future::Future, mem, time::Duration};
use async_std::task::{self, block_on};
use eachorevery::eoestruct::{StructTemplate, struct_to_json};
use eard_interp::{Interpreter, LibcoreBuilder, build_libcore, InterpreterBuilder, ProgramName, RunContext, prepare_libcore, LibcoreTemplate, GlobalBuildContext, GlobalContext, HandleStore, Return, Value, Operation};
use crate::{register::LibEoEBuilder, build_libeoe, prepare_libeoe};

#[derive(Clone)]
struct LibEoETest {
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

impl LibcoreTemplate for LibEoETest {
    fn print(&self, s: &str) {
        self.printed.lock().unwrap().push(s.to_string());
    }

    fn call_up(&self) -> Pin<Box<dyn Future<Output=Result<(),String>>>> {
        let asyncs = self.asyncs.clone();
        Box::pin(call_up_async(asyncs.clone()))
    }
}

fn printed(context: &LibEoETest) -> Vec<String> {
    mem::replace(context.printed.lock().unwrap().as_mut(),vec![])
}

fn test_string(tmpl: &StructTemplate) -> String {
    let tmpl_s = format!("{:?}",tmpl);
    let build = tmpl.build().expect("build failed");
    let json = struct_to_json(&build,None).expect("serialise failed");
    format!("{} /// {}",tmpl_s,json)
}

pub(crate) fn op_test(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let templates = gctx.patterns.lookup::<HandleStore<StructTemplate>>("eoetemplates")?;
    Ok(Box::new(move |ctx,regs| {
        let value_h = ctx.force_number(regs[1])?;
        let templates = ctx.context.get(&templates);
        let template = templates.get(value_h as usize)?;
        let out = test_string(template);
        ctx.set(regs[0],Value::String(out))?;
        Ok(Return::Sync)
    }))
}

fn prepare_interpreter() -> (Interpreter,LibcoreBuilder,LibEoEBuilder) {
    let mut builder = InterpreterBuilder::new();
    let libcore = build_libcore(&mut builder).expect("build failed");
    let libeoe = build_libeoe(&mut builder).expect("libeoe build failed");
    builder.add_operation(9999,Operation::new(op_test));
    builder.set_step_by_step(true);
    let interp = Interpreter::new(builder);
    (interp,libcore,libeoe)
}

fn run_interpreter(interp: &Interpreter, libcore: &LibcoreBuilder, libeoe: &LibEoEBuilder, part: &str) -> Vec<String> {
    let mut context = RunContext::new();
    let libcore_context = LibEoETest {
        printed: Arc::new(Mutex::new(vec![])),
        asyncs: Arc::new(Mutex::new(0))
    };
    prepare_libcore(&mut context,&libcore,libcore_context.clone());
    prepare_libeoe(&mut context,&libeoe).expect("libeoe changed");
    let program = interp.get(&ProgramName::new("smoke","smoke",1),part).expect("load failed");
    match block_on(program.run(context)) {
        Ok(_) => printed(&libcore_context),
        Err(e) => vec![e],
    }
}

#[test]
fn test_load() {
    let (mut interp,libcore,libeoe) = prepare_interpreter();
    interp.load(include_bytes!("smoke.eardo")).expect("adding");
    let out = run_interpreter(&interp,&libcore,&libeoe,"main");
    assert_eq!(vec![
        "[true] /// [true]", 
        "{\"number\": 1.0,\"string\": \"hi\"} /// {\"number\":1.0,\"string\":\"hi\"}", 
        "Aa.( a=<2.0,3.0,5.0,7.0,11.0> ) /// [2.0,3.0,5.0,7.0,11.0]",
        "Aabc.( {\"arabic\": a=<2.0,3.0,5.0,7.0,11.0>,\"roman\": b=<\"ii\",\"iii\",\"v\",\"vii\",\"xi\">,\"even\": Q[c=<true,false,false,false,false>] (true )} ) /// [{\"arabic\":2.0,\"even\":true,\"roman\":\"ii\"},{\"arabic\":3.0,\"roman\":\"iii\"},{\"arabic\":5.0,\"roman\":\"v\"},{\"arabic\":7.0,\"roman\":\"vii\"},{\"arabic\":11.0,\"roman\":\"xi\"}]",
        "Aabcd.( {\"arabic\": a=<2.0,3.0,5.0,7.0,11.0>,\"roman\": b=<\"ii\",\"iii\",\"v\",\"vii\",\"xi\">,\"type\": Q[c=<true,false,false,false,false>] (\"even\" ),\"type\": Q[d=<false,true,true,true,true>] (\"odd\" )} ) /// [{\"arabic\":2.0,\"roman\":\"ii\",\"type\":\"even\"},{\"arabic\":3.0,\"roman\":\"iii\",\"type\":\"odd\"},{\"arabic\":5.0,\"roman\":\"v\",\"type\":\"odd\"},{\"arabic\":7.0,\"roman\":\"vii\",\"type\":\"odd\"},{\"arabic\":11.0,\"roman\":\"xi\",\"type\":\"odd\"}]"
    ],out);
}
