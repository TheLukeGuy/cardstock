use rand::Rng;

const MIN_SUGGESTIONS: usize = 3;
const MAX_SUGGESTIONS: usize = 5;

pub fn gen<F>(plugin_name: &str, cmd_name: &str, mut is_taken: F) -> Vec<String>
where
    F: FnMut(&str) -> bool,
{
    let mut rng = rand::thread_rng();
    let max_suggestions = rng.gen_range::<usize, _>(MIN_SUGGESTIONS..=MAX_SUGGESTIONS);

    let mut suggestions = Vec::with_capacity(max_suggestions);
    let mut fns: Vec<_> = SUGGEST_FNS
        .iter()
        .map(|suggest_fn| {
            let ctx = SuggestCtx {
                plugin_name,
                cmd_name,
                time: 0,
            };
            (suggest_fn, ctx)
        })
        .collect();
    for _ in 0..max_suggestions {
        let range = 0..fns.len();
        if range.is_empty() {
            break;
        }
        let idx = rng.gen_range(range);

        let (&suggest_fn, ctx) = &mut fns[idx];
        let (result, limit_after_first) = suggest_fn(ctx).into_result_and_limit_after_first();
        match limit_after_first {
            Some(limit_after_first) if limit_after_first == ctx.time => {
                fns.remove(idx);
            }
            _ => ctx.time += 1,
        }

        if !is_taken(&result) {
            suggestions.push(result);
        }
    }
    suggestions
}

const SUGGEST_FNS: &[fn(&SuggestCtx) -> SuggestResult] = &[
    suggest_numerical,
    suggest_suffix,
    suggest_plugin_name,
    suggest_the,
];

fn suggest_numerical(ctx: &SuggestCtx) -> SuggestResult {
    let mut rng = rand::thread_rng();
    let value: usize = if ctx.time > MAX_SUGGESTIONS * 4 {
        rng.gen()
    } else {
        let limit = match rng.gen_range(0..10) {
            0..=3 => 999,
            4..=6 => 9999,
            7..=8 => 99999,
            9 => 999999,
            _ => panic!("impossible"),
        };
        rng.gen_range(0..=limit)
    };
    SuggestResult::Random {
        result: format!("{}{value}", ctx.cmd_name),
        limit_after_first: None,
    }
}

fn suggest_suffix(ctx: &SuggestCtx) -> SuggestResult {
    let mut rng = rand::thread_rng();
    let suffix = if rng.gen() { "_" } else { "-" }.repeat(ctx.time + 1);
    SuggestResult::Random {
        result: format!("{}{suffix}", ctx.cmd_name),
        limit_after_first: None,
    }
}

fn suggest_plugin_name(ctx: &SuggestCtx) -> SuggestResult {
    let plugin_name = ctx.plugin_name.to_lowercase();
    let result = match ctx.time {
        0 => format!("{plugin_name}{}", ctx.cmd_name),
        1 => format!("{}{plugin_name}", ctx.cmd_name),
        2 => format!("{plugin_name}-{}", ctx.cmd_name),
        3 => format!("{}-{plugin_name}", ctx.cmd_name),
        _ => panic!("impossible"),
    };
    SuggestResult::Random {
        result,
        limit_after_first: Some(3),
    }
}

fn suggest_the(ctx: &SuggestCtx) -> SuggestResult {
    SuggestResult::Constant(format!("the-{}", ctx.cmd_name))
}

struct SuggestCtx<'a> {
    pub plugin_name: &'a str,
    pub cmd_name: &'a str,
    pub time: usize,
}

enum SuggestResult {
    Constant(String),
    Random {
        result: String,
        limit_after_first: Option<usize>,
    },
}

impl SuggestResult {
    pub fn into_result_and_limit_after_first(self) -> (String, Option<usize>) {
        match self {
            SuggestResult::Constant(result) => (result, Some(0)),
            SuggestResult::Random {
                result,
                limit_after_first,
            } => (result, limit_after_first),
        }
    }
}
