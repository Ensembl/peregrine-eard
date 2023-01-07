use eard_compiler_lib::{OrBundleRepeater, PTExpression, PTCall, Constant, parse_string, ParsePosition, PTStatement, PTStatementValue};
use pest_consume::{Parser, Error, match_nodes};

#[derive(Parser)]
#[grammar = "style.pest"]
struct StyleParser;

#[allow(unused)]
type PestResult<T> = std::result::Result<T, Error<Rule>>;
#[allow(unused)]
type Node<'i> = pest_consume::Node<'i, Rule,()>;

#[pest_consume::parser]
impl StyleParser {
    fn EOI(_input: Node) -> PestResult<()> { Ok(()) }
    fn identifier(input: Node) -> PestResult<String> { Ok(input.as_str().to_string()) }
    fn bare(input: Node) -> PestResult<String> { Ok(input.as_str().to_string()) }
    fn spec(input: Node) -> PestResult<String> { Ok(input.as_str().to_string()) }

    fn number(input: Node) -> PestResult<f64> {
        input.as_str().parse::<f64>().map_err(|e| input.error(e))
    }

    fn string_inner(input: Node) -> PestResult<String> {
        parse_string(input.as_str()).map_err(|e| input.error(e))
    }

    fn string(input: Node) -> PestResult<String> {
        Ok(match_nodes!(input.into_children();
          [string_inner(s)] => s,
        ))
    }

    fn statement(input: Node) -> PestResult<(String,String)> {
        Ok(match_nodes!(input.into_children();
          [identifier(k),bare(v)] => (k,v),
          [identifier(k),string(v)] => (k,v),
          [identifier(k),number(v)] => (k,v.to_string()),
        ))
    }

    fn group(input: Node) -> PestResult<(String,Vec<(String,String)>)> {
        Ok(match_nodes!(input.into_children();
          [spec(c),statement(s)..] => (c,s.collect()),
        ))
    }

    fn document(input: Node) -> PestResult<Vec<(String,Vec<(String,String)>)>> {
        Ok(match_nodes!(input.into_children();
          [group(g)..,EOI] => g.collect(),
        ))
    }
}

fn do_parse(input: &str) -> PestResult<Vec<(String,Vec<(String,String)>)>> {
    let input = StyleParser::parse(Rule::document,&input)?.single()?;
    StyleParser::document(input)
}

fn parse(input: &str) -> Result<Vec<(String,Vec<(String,String)>)>,String> {
    do_parse(input).map_err(|e| e.to_string())
}

pub(crate) fn style_macro(expr: &[OrBundleRepeater<PTExpression>], pos: &ParsePosition, context: usize) -> Result<Vec<PTStatement>, String> {
    if expr.len() != 1 {
        return Err(format!("constant macro expects single argument"));
    }
    let value = match &expr[0] {
        OrBundleRepeater::Normal(PTExpression::Constant(Constant::String(s))) => s,
        _ => { return Err(format!("constant macro expects single, string argument")); }
    };
    let mut out = vec![];
    for (spec,mut kvs) in parse(value)? {
        kvs.sort();
        let keys = kvs.iter().map(|x| 
            PTExpression::Constant(Constant::String(x.0.to_string()))
        ).collect::<Vec<_>>();
        let values = kvs.iter().map(|x| 
            PTExpression::Constant(Constant::String(x.1.to_string()))
        ).collect::<Vec<_>>();
        out.push(PTStatement { 
            value: PTStatementValue::Expression(PTExpression::Call(PTCall { 
                name: "style".to_string(), 
                args: vec![
                    OrBundleRepeater::Normal(PTExpression::Constant(Constant::String(spec.clone()))),
                    OrBundleRepeater::Normal(PTExpression::FiniteSequence(keys)),
                    OrBundleRepeater::Normal(PTExpression::FiniteSequence(values)),
                ], 
                is_macro: false
            })),
            position: pos.clone(), 
            context 
        });
    }
    Ok(out)
}
