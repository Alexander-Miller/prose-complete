use emacs::{defun, plugin_is_GPL_compatible, Env, IntoLisp, Result, Value};
use once_cell::sync::OnceCell;
use std::str;
use trie_rs::{Trie, TrieBuilder};

plugin_is_GPL_compatible!();

const LIMIT: usize = 1000;
const WORDS: &'static str = include_str!("words");
static INSTANCE: OnceCell<Trie<u8>> = OnceCell::new();

#[emacs::module]
fn init(env: &Env) -> Result<Value> {
    let mut builder: TrieBuilder<u8> = TrieBuilder::new();
    for line in WORDS.lines() {
        builder.push(line);
    }
    match INSTANCE.set(builder.build()) {
        Ok(_) => env.message("Prose-Complete module loaded"),
        Err(_) => on_error(env, "Failed to set trie instance"),
    }
}

#[defun]
fn lookup<'a>(env: &'a Env, str: String) -> Result<Value> {
    let trie = match INSTANCE.get() {
        Some(it) => it,
        None => return on_error(env, "Failed to aquire trie instance"),
    };

    if str.is_empty() {
        return env.call("list", &[]);
    }

    let strings_found: Vec<String> = trie
        .predictive_search(str)
        .into_iter()
        .map(|u8s| String::from_utf8(u8s).unwrap())
        .collect();

    let result_strs: Vec<Value> = match strings_found.len() > LIMIT {
        true => strings_to_values_prefixes_only(env, strings_found),
        false => strings_to_values(env, strings_found),
    };

    return env.call("list", &result_strs);
}

fn on_error<'a>(env: &'a Env, message: &str) -> Result<Value<'a>> {
    return env.call("error", &[message.into_lisp(env)?]);
}

#[inline(always)]
fn strings_to_values(env: &Env, strings: Vec<String>) -> Vec<Value> {
    strings.iter().map(|s| s.into_lisp(env).unwrap()).collect()
}

#[inline(always)]
fn strings_to_values_prefixes_only(env: &Env, strings: Vec<String>) -> Vec<Value> {
    strings
        .iter()
        .filter(|item| {
            !strings
                .iter()
                .any(|other| !(*item).eq(other) && item.starts_with(other))
        })
        .map(|s| s.into_lisp(env).unwrap())
        .take(LIMIT)
        .collect()
}
