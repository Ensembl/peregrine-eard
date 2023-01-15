use crate::{GlobalBuildContext, GlobalContext, Return, Value};

fn process_gap(out: &mut Vec<(f64,f64)>, start: f64, end: f64, gap_start: Option<f64>, gap_end: Option<f64>) {
    let gap_end = if let Some(gap_end) = gap_end {
        if gap_end <= start { return; } // gap has end and ends before start
        gap_end.min(end) // gap has end after our start, trim end if necessary
    } else {
        end // gap is endless on right, trim
    };
    let gap_start = if let Some(gap_start) = gap_start {
        if gap_start >= end { return; } // gap has start and starts after end
        gap_start.max(start) // gap has start after out start, trim start if necessary
    } else {
        start // gap is endless on left, trim
    };
    if gap_end >= gap_start {
        out.push((gap_start,gap_end));
    }
}

fn gaps_one(start: f64, end: f64, mut blocks: Vec<(f64,f64)>) -> Vec<(f64,f64)> {
    blocks.sort_by_key(|(a,b)| a.partial_cmp(b).unwrap());
    /* Call our block start ends (a0,b0), (a1,b1), (a2,b2) etc (an,bn)
     * Our gaps are then (-INF,a0), (b0,a1), (b1,a2),  (bn,+INF).
     * Pass these to process_gap().
     */
     let mut out = vec![];
    let mut prev_start = None;
    for (block_start,block_end) in blocks.iter() {
        process_gap(&mut out, start,end,prev_start,Some(*block_start));
        prev_start = Some(*block_end);
    }
    process_gap(&mut out, start,end,prev_start,None);
    out
}

/* Given (start,end), chop out (block_start,block_end) where block_index maps blocks to correct
 * tuple. Used, for example, for chopping exons out of transcripts to give list of introns.
 * Sounds niche, but supririsingly valuable.
 */

fn gaps(starts: &[f64], ends: &[f64], block_starts: &[f64], block_ends: &[f64], block_indexes: &[usize]) -> (Vec<f64>,Vec<f64>,Vec<usize>) {
    // TODO check lens etc
    /* Assemble the blocks by index */
    let mut blocks = vec![vec![];starts.len()];
    for (index,(start,end)) in block_indexes.iter().zip(block_starts.iter().zip(block_ends.iter())) {
        blocks[*index].push((*start,*end));
    }
    /* Process each separately */
    let mut out_start = vec![];
    let mut out_end = vec![];
    let mut out_index = vec![];
    for (index,((start,end),blocks)) in starts.iter().zip(ends.iter()).zip(blocks.drain(..)).enumerate() {
       for (gap_start, gap_end) in gaps_one(*start,*end,blocks) {
           out_start.push(gap_start);
           out_end.push(gap_end);
           out_index.push(index);
       }
    }
    (out_start,out_end,out_index)
}

pub(crate) fn op_gaps(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        let starts = ctx.force_finite_number(regs[3])?;
        let ends = ctx.force_finite_number(regs[4])?;
        let block_starts = ctx.force_finite_number(regs[5])?;
        let block_ends = ctx.force_finite_number(regs[6])?;
        let block_indexes = ctx.force_finite_number(regs[7])?;
        let block_indexes = block_indexes.iter().map(|x| *x as usize).collect::<Vec<_>>();
        let (out_start,out_end,out_indexes) = gaps(starts,ends,block_starts,block_ends,&block_indexes);
        ctx.set(regs[0],Value::FiniteNumber(out_start))?;
        ctx.set(regs[1],Value::FiniteNumber(out_end))?;
        ctx.set(regs[2],Value::FiniteNumber(out_indexes.iter().map(|x| *x as f64).collect()))?;
        Ok(Return::Sync)
    }))
}
