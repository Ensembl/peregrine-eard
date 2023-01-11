use eard_compiler_lib::parse_string;
use pest_consume::{Parser, Error, match_nodes};

use crate::parsetree::{StructExpr, VarType};

#[derive(Parser)]
#[grammar = "struct.pest"]
struct StyleParser;

#[allow(unused)]
type PestResult<T> = std::result::Result<T, Error<Rule>>;
#[allow(unused)]
type Node<'i> = pest_consume::Node<'i, Rule,()>;

#[pest_consume::parser]
impl StyleParser {

    fn expr(input: Node) -> PestResult<StructExpr> {
        Ok(match_nodes!(input.into_children();
            [object(b)] => b,
            [array(a)] => a,
            [string(s)] => StructExpr::String(s),
            [number(n)] => n,
            [boolean(b)] => StructExpr::Boolean(b),
            [null(n)] => n,
            [var(v)] => v,
            [cond(c)] => c,
            [all(a)] => a,
        ))
    }


    fn null(input: Node) -> PestResult<StructExpr> { Ok(StructExpr::Null) }

    fn vartype(input: Node) -> PestResult<VarType> {
        Ok(match input.as_str() {
            "s" => VarType::String,
            "n" => VarType::Number,
            "b" => VarType::Boolean,
            _ => unreachable!()
        })
    }

    fn string_inner(input: Node) -> PestResult<String> {
        parse_string(input.as_str()).map_err(|e| input.error(e))
    }

    fn string(input: Node) -> PestResult<String> {
        Ok(match_nodes!(input.into_children();
          [string_inner(s)] => s
        ))
    }

    fn boolean(input: Node) -> PestResult<bool> {
        Ok(match input.as_str() {
            "false" => false,
            "true" => true,
            _ => unreachable!()
        })
    }

    fn varnum(input: Node) -> PestResult<usize> {
        Ok(input.as_str().parse::<usize>().map_err(|e| input.error(e))?)
    }

    fn var(input: Node) -> PestResult<StructExpr> {
        Ok(match_nodes!(input.into_children();
          [varnum(v),vartype(t)] => StructExpr::Var(None,t,v)
        ))
    }

    fn array(input: Node) -> PestResult<StructExpr> {
        Ok(match_nodes!(input.into_children();
          [expr(x)..] => StructExpr::Array(x.collect())
        ))
    }

    fn pair(input: Node) -> PestResult<(String,StructExpr)> {
        Ok(match_nodes!(input.into_children();
          [string(s),expr(x)] => (s,x)
        ))
    }

    fn object(input: Node) -> PestResult<StructExpr> {
        Ok(match_nodes!(input.into_children();
          [pair(p)..] => StructExpr::Object(p.collect())
        ))
    }

    fn cond(input: Node) -> PestResult<StructExpr> {
        Ok(match_nodes!(input.into_children();
          [varnum(p),expr(x)] => StructExpr::Cond(p,Box::new(x))
        ))
    }

    fn all(input: Node) -> PestResult<StructExpr> {
        Ok(match_nodes!(input.into_children();
          [varnum(p)..,expr(x)] => StructExpr::All(0,p.collect(),Box::new(x))
        ))
    }

    fn number(input: Node) -> PestResult<StructExpr> {
        Ok(StructExpr::Number(input.as_str().parse::<f64>().map_err(|e| input.error(e))?))
    }

    fn EOI(_input: Node) -> PestResult<()> { Ok(()) }

    fn document(input: Node) -> PestResult<StructExpr> {
        Ok(match_nodes!(input.into_children();
            [expr(x),_EOI] => x
        ))
    }
}

fn do_parse(input: &str) -> PestResult<StructExpr> {
    let input = StyleParser::parse(Rule::document,&input)?.single()?;
    StyleParser::document(input)
}

pub(crate) fn parse(input: &str) -> Result<StructExpr,String> {
    do_parse(input).map_err(|e| e.to_string())
}
